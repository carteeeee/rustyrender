use ocl::{Buffer, ProQue};
use sdl2::rect::Rect;
use sdl2::video::Window;
use std::ops::{Add, Div, Mul, Sub};
// All of the rendering is done on the gpu using this kernel. See the below `render` function for
// more details.
static KERNEL_SRC: &'static str = r#"
    __kernel void render(
                __global float const* const tris,
                __global uint* const out,
                __private uint const width)
    {
        uint const idx = get_global_id(0);
        out[idx/2] = 255;
    }
"#;

// `Vec3f` implementation, this is basically the type used for everything from 3d rotation to
// position. Addition and subtraction between Vec3fs is implemented and multiplication and division
// are only implemented with an f32.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3f {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3f {
    pub fn x(&self) -> f32 {
        self.x
    }
    pub fn y(&self) -> f32 {
        self.y
    }
    pub fn z(&self) -> f32 {
        self.z
    }

    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl Add<Vec3f> for Vec3f {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub<Vec3f> for Vec3f {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul<f32> for Vec3f {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Div<f32> for Vec3f {
    type Output = Self;

    fn div(self, rhs: f32) -> Self {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

// `Triangle` implementation, this is a simple struct for storing the data of a triangle, I may add
// color soon but right now this can do basic computation of triangle-related things.
#[derive(Copy, Clone, Debug)]
pub struct Triangle {
    v1: Vec3f,
    v2: Vec3f,
    v3: Vec3f,
}

impl Triangle {
    pub fn from_points(v1: Vec3f, v2: Vec3f, v3: Vec3f) -> Self {
        Self { v1, v2, v3 }
    }

    // The star of the show, the `point_in_triangle` function. This tells wether the ray has
    // reached the triangle, and returns false if no and true if yes. If you can optimize this even
    // more, please do!
    fn point_in_triangle(&self, rayx: f32, rayy: f32, rayz: f32) -> bool {
        // compute normals for the following tris:
        // point, v1, v2
        // point, v2, v3
        // point, v3, v1
        let u1x = self.v1.x - rayx;
        let u1y = self.v1.y - rayy;
        let u1z = self.v1.z - rayz;

        let v1x = self.v2.x - rayx;
        let v1y = self.v2.y - rayy;
        let v1z = self.v2.z - rayz;

        let n1x = u1y * v1z - u1z * v1y;
        let n1y = u1z * v1x - u1x * v1z;
        let n1z = u1x * v1y - u1y * v1x;

        let u2x = self.v2.x - rayx;
        let u2y = self.v2.y - rayy;
        let u2z = self.v2.z - rayz;

        let v2x = self.v3.x - rayx;
        let v2y = self.v3.y - rayy;
        let v2z = self.v3.z - rayz;

        let n2x = u2y * v2z - u2z * v2y;
        let n2y = u2z * v2x - u2x * v2z;
        let n2z = u2x * v2y - u2y * v2x;

        let u3x = self.v3.x - rayx;
        let u3y = self.v3.y - rayy;
        let u3z = self.v3.z - rayz;

        let v3x = self.v1.x - rayx;
        let v3y = self.v1.y - rayy;
        let v3z = self.v1.z - rayz;

        let n3x = u3y * v3z - u3z * v3y;
        let n3y = u3z * v3x - u3x * v3z;
        let n3z = u3x * v3y - u3y * v3x;

        // determine if a point is within the triangle
        let d1 = n1x * n2x + n1y * n2y + n1z * n2z;
        if d1 < 0.0 {
            return false;
        }

        let d2 = n1x * n3x + n1y * n3y + n1z * n3z;
        if d2 < 0.0 {
            return false;
        }

        true
    }
}

// The `Camera` struct doesn't do much, it just stores the camera's info in a nice little package.
pub struct Camera {
    pos: Vec3f, // x y z
    rot: Vec3f, // pitch roll yaw
    fov: f32,
}

// There once was a struct called `Geometry` but I have killed him because he was useless.
fn origin_to_camera(geometry: &Vec<Triangle>, camera: &Camera) -> Vec<Triangle> {
    let origin = camera.pos;
    let _rotation = camera.rot;
    let mut newgeo = geometry.clone();
    for triangle in &mut newgeo {
        triangle.v1 = triangle.v1 - origin;
        triangle.v2 = triangle.v2 - origin;
        triangle.v3 = triangle.v3 - origin;
    }
    newgeo
}

// not what it sounds like but it converts the geomtry (a buncha triangles) into a vector with the
// triangles v1x v1y v1z v2x v2y and so on consecutively in the vector so that the gpu kernel has a
// nice little vector of f32s to take in
fn geometry_to_points(geometry: &Vec<Triangle>) -> Vec<f32> {
    let mut output: Vec<f32> = Vec::new();
    for triangle in geometry {
        output.extend([
            triangle.v1.x,
            triangle.v1.y,
            triangle.v1.z,
            triangle.v2.x,
            triangle.v2.y,
            triangle.v2.z,
            triangle.v3.x,
            triangle.v3.y,
            triangle.v3.z,
        ]);
    }
    output
}

// using a struct for this because i need to store the ocl proque and buffers and maybe kernel too
pub struct Renderer {
    proque: ProQue,
    tribuf: Buffer<f32>,
    outbuf: Buffer<u8>,
}

impl Renderer {
    pub fn new() -> Self {
        let proque = ProQue::builder()
            .src(KERNEL_SRC)
            .dims(1 << 18)
            .build()
            .expect("could not build proque");
        let tribuf = proque
            .create_buffer::<f32>()
            .expect("could not create triangle buffer");
        let outbuf = proque
            .create_buffer::<u8>()
            .expect("could not create output buffer");

        Self {
            proque,
            tribuf,
            outbuf,
        }
    }

    // The `render` function will take your `Window`, `EventPump`, `Vec<Triangle>` (geometry), and `Camera` and will
    // draw directly onto the window without a renderer. This saves time (maybe?) because it doesn't
    // need a 2d rendering engine such as opengl running underneath. This is basically the entire code
    // for the rendering engine and is very delicate, so if you're opening a pull request, make sure
    // that you don't fuck up this function! There's also a lot of development shit here too, don't
    // delete that because otherwise I'll forget everything about this function
    //
    // UPDATE: i am now adding gpu support using the `ocl` crate. forget everything you konw about this
    // function because it's bouta get the craziest glowup ever.
    pub fn render(
        &mut self,
        window: &mut Window,
        event_pump: &sdl2::EventPump,
        geometry: &Vec<Triangle>,
        camera: &Camera,
    ) -> Result<(), String> {
        let mut surface = window.surface(event_pump)?;
        let newgeo = origin_to_camera(&geometry, &camera);
        let geopoints = geometry_to_points(&newgeo);

        let sw = surface.width();
        let sh = surface.height();

        //self.proque.set_dims(sh * sw);

        self.tribuf
            .write(&geopoints)
            .enq()
            .expect("could not write triangles to buffer");

        let kernel = self
            .proque
            .kernel_builder("render")
            .arg(&self.tribuf)
            .arg(&self.outbuf)
            .arg(&sw)
            .build()
            .expect("could not build kernel");

        unsafe {
            kernel.enq().expect("could not enque kernel"); // wow, unsafe code and expect in one line!
                                                           // so much memory safety here
        }

        let mut outvec = vec![0u8; self.outbuf.len()];
        self.outbuf
            .read(&mut outvec)
            .enq()
            .expect("could not read outbuffer");

        for i in 0..sw * sh {
            let x = (i % sw) as i32;
            let y = (i / sw) as i32;
            let out = outvec[i as usize];
            let _ = surface.fill_rect(Rect::new(x, y, 1, 1), (out, out, out).into());
        }
        /*
        for y in 0..sh as i32 {
            for x in 0..sw as i32 {
                let index = (y * sw as i32 + x) as usize;
                let out = outvec[index];
                let _ = surface.fill_rect(Rect::new(x, y, 1, 1), (out, out, out).into());
            }
        }
        */

        println!("debug");
        surface.finish()
    }
}

// Behold, the shitty cargo tests that I have made.
#[cfg(test)]
mod tests {
    use super::*;
    use sdl2::event::Event;
    use sdl2::keyboard::Keycode;
    use std::time::Duration;
    //use std::time::Instant;

    // Tests the point in triangle function using a predefined triangle and expected result.
    #[test]
    fn point_in_triangle_test() {
        let v1 = Vec3f::new(0.0, 0.0, 0.0);
        let v2 = Vec3f::new(0.0, 0.0, 2.0);
        let v3 = Vec3f::new(2.0, 0.0, 0.0);
        let t = Triangle::from_points(v1, v2, v3);
        assert!(t.point_in_triangle(1.0, 0.0, 1.0));
    }

    // I know that I'm not supposed to be running sdl2 type shit here, this should actually
    // probably be an exmaple, but there's really no other way of testing the rendering function.
    // This will also catch any crashes that may occur due to OOM or other shit that I forgot
    // about.
    #[test]
    fn render_test() -> Result<(), String> {
        let width = 500;
        let height = 500;
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;

        let mut window = video_subsystem
            .window("rustyrender test", width, height)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;
        let mut event_pump = sdl_context.event_pump()?;

        let v11 = Vec3f::new(0.0, 0.0, 1.0);
        let v21 = Vec3f::new(1.0, 0.0, 1.0);
        let v31 = Vec3f::new(0.0, 1.0, 1.0);

        let triangle1 = Triangle::from_points(v11, v21, v31);

        let v12 = Vec3f::new(1.0, 0.0, 1.0);
        let v22 = Vec3f::new(0.0, 1.0, 1.0);
        let v32 = Vec3f::new(1.0, 1.0, 1.0);

        let triangle2 = Triangle::from_points(v12, v22, v32);

        let geometry = vec![triangle1, triangle2];

        let mut camera = Camera {
            pos: Vec3f::new(5.0, 0.0, -5.0),
            rot: Vec3f::new(65.0, 0.0, 46.0),
            fov: 50.0,
        };

        let mut renderer = Renderer::new();

        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    _ => {}
                }
            }
            camera.pos.z -= 1.0;

            //let now = Instant::now();
            renderer.render(&mut window, &event_pump, &geometry, &camera)?;
            //let elapsed = now.elapsed();
            //println!("Elapsed: {:.2?}", elapsed);
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }

        Ok(()) // wahoo!!
    }
}
