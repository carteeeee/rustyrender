use ocl::{Buffer, ProQue};
use sdl2::rect::Rect;
use sdl2::video::Window;
use std::ops::{Add, Div, Mul, Sub};
// all of the rendering is done on the gpu using this kernel. see the below `render` function for
// more details.
static KERNEL_SRC: &'static str = r#"
    __kernel void render(
                __global float3 const* const tris,
                __global uchar* const out,
                __private float const fov,
                __private uint const width,
                __private uint const height,
                __private ulong const numtris)
    {

        uint const idx = get_global_id(0);
        float const x = idx % width;
        float const y = idx / width;
        float2 const edir = {
            (x / width - 0.5) * fov,
            (y / height - 0.5) * fov
        };
        float2 const edirr = radians(edir);
        float3 const rdir = {
            cos(edirr.x) * cos(edirr.y),
            sin(edirr.x) * cos(edirr.y),
            sin(edirr.y)
        };

        bool stop = false;
        uint end = 0;
        for (uint i = 0; (i < 100) && !stop; i++) {
            float3 const raypos = rdir * (i / 0.1f);
            end = i;
            for (uint t = 0; (t < numtris) && !stop; t++) {
                float3 const tv1 = tris[t];
                float3 const tv2 = tris[t + 1];
                float3 const tv3 = tris[t + 2];

                float3 u1 = tv1 - raypos;
                float3 v1 = tv2 - raypos;
                float3 n1 = cross(u1, v1);

                float3 u2 = tv2 - raypos;
                float3 v2 = tv3 - raypos;
                float3 n2 = cross(u2, v2);

                float3 u3 = tv3 - raypos;
                float3 v3 = tv1 - raypos;
                float3 n3 = cross(u3, v3);

                float d1 = dot(n1, n2);
                float d2 = dot(n1, n3);

                if (!(d1 < 0.0f) && !(d2 < 0.0f)) {
                    stop = true;
                    break;
                }
            }
        }
        out[idx] = end;
    }
"#;

// vector oh yeah!!! this is basically the type used for everything from 3d rotation to
// position. addition and subtraction between Vec3fs is implemented and multiplication and division
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

// wow triangles so cool!! this is a simple struct for storing the data of a triangle, i may add
// color soon:tm:
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
}

// this only exists to store the camera's info in a nice little package.
pub struct Camera {
    pos: Vec3f, // x y z
    rot: Vec3f, // pitch roll yaw
    fov: f32,
}

// there once was a struct called `Geometry` but I have killed him because he was fucking useless.
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

// using a struct for this because i need to store the ocl proque and buffers
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

    // the `render` function will take your `Window`, `EventPump`, `Vec<Triangle>` (geometry), and `Camera` and will
    // draw directly onto the window without a renderer. this saves time (maybe?) because it doesn't
    // need a 2d rendering engine such as opengl running underneath.
    //
    // MAJOR update: i have now added gpu support. this was very painful and probably is very slow
    // so please please PLEASE help make this faster!!!
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

        self.proque.set_dims(sh * sw);

        self.tribuf
            .write(&geopoints)
            .enq()
            .expect("could not write triangles to buffer");

        let kernel = self
            .proque
            .kernel_builder("render")
            .arg(&self.tribuf)
            .arg(&self.outbuf)
            .arg(&camera.fov)
            .arg(&sw)
            .arg(&sh)
            .arg(&geometry.len())
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

        surface.finish()
    }
}

// behold, the shitty cargo tests i made.
#[cfg(test)]
mod tests {
    use super::*;
    use sdl2::event::Event;
    use sdl2::keyboard::Keycode;
    use std::time::Duration;
    //use std::time::Instant;

    // yeahhhh i'm probably not supposed to add sdl2 code in my tests but fuck it i don't give two
    // shits.
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

        let geometry = vec![/*triangle1, */ triangle2];

        let mut camera = Camera {
            pos: Vec3f::new(5.0, 0.0, -10.0),
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
            camera.pos.x -= 1.0;

            //let now = Instant::now();
            renderer.render(&mut window, &event_pump, &geometry, &camera)?;
            //let elapsed = now.elapsed();
            //println!("Elapsed: {:.2?}", elapsed);
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }

        Ok(()) // wahoo!!
    }
}
