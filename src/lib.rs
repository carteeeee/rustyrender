use sdl2::pixels::Color;
use sdl2::video::Window;

// `Vec3f` implementation, this is basically the type used for everything from 3d rotation to
// position. It only has the functions it needs, so I usually add functions to it as I go instead
// of adding basic arithematic ahead of time like + - * /
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

    fn sub(&self, point: &Self) -> Self {
        Self {
            x: self.x - point.x,
            y: self.y - point.y,
            z: self.z - point.z,
        }
    }

    fn mult(&self, num: f32) -> Self {
        Self {
            x: self.x * num,
            y: self.y * num,
            z: self.z * num,
        }
    }
}

// `Triangle` implementation, this is a simple struct for storing the data of a triangle, I may add
// color soon but right now this can do basic computation of triangle-related things. Also, this is
// used to store three normals lmao.
#[derive(Copy, Clone, Debug)]
pub struct Triangle {
    v1: Vec3f,
    v2: Vec3f,
    v3: Vec3f,
}

impl Triangle {
    fn from_points(v1: Vec3f, v2: Vec3f, v3: Vec3f) -> Self {
        Self { v1, v2, v3 }
    }

    // The star of the show, the `point_in_triangle` function. This tells wether the ray has
    // reached the triangle, and returns false if no and true if yes. If you can optimize this even
    // more, please do! Right now it's taking approx 100ms to render a full frame at 200x200. With
    // optimizations enabled (release mode), it takes 18ns to run and with debug it takes 22ns to
    // run.
    fn point_in_triangle(&self, point: Vec3f) -> bool {
        // compute normals for the following tris:
        // point, v1, v2
        // point, v2, v3
        // point, v3, v1
        let u1x = self.v1.x - point.x;
        let u1y = self.v1.y - point.y;
        let u1z = self.v1.z - point.z;

        let v1x = self.v2.x - point.x;
        let v1y = self.v2.y - point.y;
        let v1z = self.v2.z - point.z;

        let n1x = u1y * v1z - u1z * v1y;
        let n1y = u1z * v1x - u1x * v1z;
        let n1z = u1x * v1y - u1y * v1x;

        let u2x = self.v2.x - point.x;
        let u2y = self.v2.y - point.y;
        let u2z = self.v2.z - point.z;

        let v2x = self.v3.x - point.x;
        let v2y = self.v3.y - point.y;
        let v2z = self.v3.z - point.z;

        let n2x = u2y * v2z - u2z * v2y;
        let n2y = u2z * v2x - u2x * v2z;
        let n2z = u2x * v2y - u2y * v2x;

        let u3x = self.v3.x - point.x;
        let u3y = self.v3.y - point.y;
        let u3z = self.v3.z - point.z;

        let v3x = self.v1.x - point.x;
        let v3y = self.v1.y - point.y;
        let v3z = self.v1.z - point.z;

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

// The `Geometry` struct contains the geometry but it also transforms the geometry to having the
// camera as the origin.

#[derive(Clone)]
pub struct Geometry {
    triangles: Vec<Triangle>,
}

impl Geometry {
    fn origin_to_camera_and_scale(&self, camera: &Camera) -> Self {
        let origin = camera.pos;
        let _rotation = camera.rot;
        let mut newgeo = self.clone();
        for triangle in &mut newgeo.triangles {
            triangle.v1 = triangle.v1.sub(&origin);
            triangle.v2 = triangle.v2.sub(&origin);
            triangle.v3 = triangle.v3.sub(&origin);
        }
        newgeo
    }
}

// The `render` function will take your `Window`, `EventPump`, `Geometry`, and `Camera` and will
// draw directly onto the window without a renderer. This saves time (maybe?) because it doesn't
// need a 2d rendering engine such as opengl running underneath. This is basically the entire code
// for the rendering engine and is very delicate, so if you're opening a pull request, make sure
// that you don't fuck up this function! There's also a lot of development shit here too, don't
// delete that because otherwise I'll forget everything about this function.
pub fn render(
    window: &mut Window,
    event_pump: &sdl2::EventPump,
    geometry: &Geometry,
    camera: &Camera,
    ticks: f32,
) -> Result<(), String> {
    let mut surface = window.surface(event_pump)?;
    let srect = surface.rect();
    let _newgeo = geometry.origin_to_camera_and_scale(&camera);

    let sw = surface.width();
    let pixel_format = surface.pixel_format_enum();
    let surface_data = surface.without_lock_mut().unwrap();
    let bpp = pixel_format.byte_size_per_pixel();

    let v11 = Vec3f::new(0.0, 0.0, 1.0);
    let v21 = Vec3f::new(1.0, 0.0, 1.0);
    let v31 = Vec3f::new(0.0, 1.0, 1.0);

    let triangle1 = Triangle::from_points(v11, v21, v31);

    for x in 0..srect.width() {
        for y in 0..srect.height() {
            let pitch =
                ((y as f32 / srect.height() as f32 - 0.5) * camera.fov + 40.0 + ticks).to_radians();
            let yaw = ((x as f32 / srect.height() as f32 - 0.5) * camera.fov + 180.0).to_radians();
            let dir = Vec3f::new(
                yaw.cos() * pitch.cos(),
                yaw.sin() * pitch.cos(),
                pitch.sin(),
            )
            .mult(0.1);
            let index = (y * sw + x) as usize * bpp;

            let mut res: u8 = 100;
            'rtx: for z in 0..100 {
                let raypos = dir.mult(z as f32);
                //for i in 0..newgeo.triangles.len() {
                let a = triangle1.point_in_triangle(raypos);
                if a {
                    res = z;
                    break 'rtx;
                }
                //}
                if z == 100 {
                    res = z;
                    break 'rtx;
                }
            }
            surface_data[index] = res;
            surface_data[index + 1] = res;
            surface_data[index + 2] = res;
        }
    }
    println!("help me");
    surface.finish()
}

// Behold, the shitty cargo tests that I have made.
#[cfg(test)]
mod tests {
    use super::*;
    use sdl2::event::Event;
    use sdl2::keyboard::Keycode;
    use std::time::Duration;
    use std::time::Instant;

    // Tests the point in triangle function using a predefined triangle and expected result.
    #[test]
    fn point_in_triangle_test() {
        let v1 = Vec3f::new(0.0, 0.0, 0.0);
        let v2 = Vec3f::new(0.0, 0.0, 2.0);
        let v3 = Vec3f::new(2.0, 0.0, 0.0);
        let t = Triangle::from_points(v1, v2, v3);
        let p = Vec3f::new(1.0, 0.0, 1.0);
        assert!(t.point_in_triangle(p));
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

        println!("test");
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

        let geometry = Geometry {
            triangles: vec![triangle1, triangle2],
        };

        let mut camera = Camera {
            pos: Vec3f::new(5.0, 0.0, -5.0),
            rot: Vec3f::new(65.0, 0.0, 46.0),
            fov: 50.0,
        };

        let mut ticks = 0.0;

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
            ticks += 1.0;

            let now = Instant::now();
            render(&mut window, &event_pump, &geometry, &camera, ticks)?;
            let elapsed = now.elapsed();
            println!("Elapsed: {:.2?}", elapsed);
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }

        Ok(()) // wahoo!!
    }
}
