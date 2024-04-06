use sdl2::pixels::Color;
use sdl2::rect::Rect;
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

    fn cross(&self, target: &Self) -> Self {
        Self {
            x: self.y * target.z - self.z * target.y,
            y: self.z * target.x - self.x * target.z,
            z: self.x * target.y - self.y * target.x,
        }
    }

    fn dot(&self, target: &Self) -> f32 {
        self.x * target.x + self.y * target.y + self.z * target.z
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
// color soon but right now this can do basic computation of triangle-related things.
#[derive(Clone, Debug)]
pub struct Triangle {
    v1: Vec3f,
    v2: Vec3f,
    v3: Vec3f,
}

impl Triangle {
    fn from_points(v1: Vec3f, v2: Vec3f, v3: Vec3f) -> Self {
        Self { v1, v2, v3 }
    }

    fn normal(&self) -> Vec3f {
        let u = self.v2.sub(&self.v1);
        let v = self.v3.sub(&self.v1);
        u.cross(&v)
    }

    fn multiple_normals(&self, point: Vec3f) -> Self {
        let t1 = Self::from_points(point, self.v1, self.v2);
        let t2 = Self::from_points(point, self.v2, self.v3);
        let t3 = Self::from_points(point, self.v3, self.v1);

        Self {
            v1: t1.normal(),
            v2: t2.normal(),
            v3: t3.normal(),
        }
    }

    fn point_in_triangle(&self, point: Vec3f) -> bool {
        let n = self.multiple_normals(point);

        if n.v1.dot(&n.v2) < 0.0 {
            return false;
        }
        if n.v1.dot(&n.v3) < 0.0 {
            return false;
        }

        true
    }

    fn new_origin(&mut self, origin: Vec3f) {
        self.v1 = self.v1.sub(&origin);
        self.v2 = self.v2.sub(&origin);
        self.v3 = self.v3.sub(&origin);
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
pub struct Geometry {
    triangles: Vec<Triangle>,
}

impl Geometry {
    fn origin_to_camera(&mut self, camera: &Camera) {
        let origin = camera.pos;
        let rotation = camera.rot;

        for triangle in &mut self.triangles {
            triangle.new_origin(origin);
        }
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

    surface.fill_rect(srect, Color::RGB(0, 0, 0))?;

    for x in 0..srect.width() {
        for y in 0..srect.height() {
            let pitch = ((y as f32 / srect.height() as f32 - 0.5) * camera.fov).to_radians();
            let yaw = ((x as f32 / srect.height() as f32 - 0.5) * camera.fov + ticks + 180.0)
                .to_radians();
            let dir = Vec3f::new(
                //-yaw.cos() * pitch.sin() * roll.sin() - yaw.sin() * roll.cos(),
                //-yaw.sin() * pitch.sin() * roll.sin() + yaw.cos() * roll.cos(),
                //pitch.cos() * roll.sin(),
                yaw.cos() * pitch.cos(),
                yaw.sin() * pitch.cos(),
                pitch.sin(),
            )
            .mult(0.25);
            //println!("{:?}", dir);
            //println!("{:?}", Vec3f::new(pitch, roll, yaw));
            let mut res: u8 = 100;
            'rtx: for z in 0..100 {
                let raypos = dir.mult(z as f32);
                for triangle in &geometry.triangles {
                    if triangle.point_in_triangle(raypos) {
                        res = z;
                        break 'rtx;
                    }
                }
                if z == 100 {
                    res = z;
                    break 'rtx;
                }
            }
            surface.fill_rect(
                Rect::new(x as i32, y as i32, 1, 1),
                Color::RGB(res, res, res),
            )?;
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

    // Tests triangle normal calculation using a predefined triangle and expected normal.
    #[test]
    fn triangle_normal_test() {
        let v1 = Vec3f::new(0.0, 0.0, 0.0);
        let v2 = Vec3f::new(0.0, 0.0, 2.0);
        let v3 = Vec3f::new(2.0, 0.0, 0.0);
        let t = Triangle::from_points(v1, v2, v3);
        let n = t.normal();
        println!("{:?}", n);
        assert_eq!(n, Vec3f::new(0.0, 4.0, 0.0));
    }

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

        let mut window = video_subsystem
            .window("rustyrender test", width, height)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;
        let timer = sdl_context.timer()?;
        let mut event_pump = sdl_context.event_pump()?;

        let v1 = Vec3f::new(0.0, 10.0, 10.0);
        let v2 = Vec3f::new(1.5, -0.2, 1.5);
        let v3 = Vec3f::new(0.4, -1.3, -0.7);

        let triangle = Triangle::from_points(v1, v2, v3);
        let mut geometry = Geometry {
            triangles: vec![triangle],
        };

        let camera = Camera {
            pos: Vec3f::new(7.0, -6.0, 4.0),
            rot: Vec3f::new(65.0, 0.0, 46.0),
            fov: 40.0,
        };

        geometry.origin_to_camera(&camera);
        let mut ticks: f32 = 0.0;
        println!("{:?}", geometry.triangles[0].v1);
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
            //let ticks = timer.ticks() as i32;
            ticks -= 10.0;
            render(&mut window, &event_pump, &geometry, &camera, ticks)?;
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }

        Ok(()) // wahoo!!
    }
}
