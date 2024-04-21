use crate::types::*;
use ocl::prm::Float3;
use ocl::{Buffer, ProQue};
use sdl2::rect::Rect;
use sdl2::video::Window;

// all the rendering stuffs is done in kernel.cl and then it goes to the cpu for other stuff idk
// man
static KERNEL_SRC: &'static str = include_str!("kernel.cl");

// there once was a struct called `Geometry` but i have killed him because he was fucking useless.
fn origin_to_camera(geometry: &Vec<Triangle>, camera: &Camera) -> Vec<Triangle> {
    let origin = camera.pos;
    let rotation = camera.rot;

    let rm = RotationMatrix::from_euler(rotation);
    let mut newgeo = geometry.clone();
    for triangle in &mut newgeo {
        triangle.v1 = rm.rotate_vector(triangle.v1 - origin);
        triangle.v2 = rm.rotate_vector(triangle.v2 - origin);
        triangle.v3 = rm.rotate_vector(triangle.v3 - origin);
    }
    newgeo
}

// converts your geometry into a vector of 3 component float (f32 in rust) vectors with each vertex
// consecutive ig.
fn geometry_to_points(geometry: &Vec<Triangle>) -> Vec<Float3> {
    let mut output: Vec<Float3> = Vec::new();
    for triangle in geometry {
        output.extend([
            Float3::new(triangle.v1.x, triangle.v1.y, triangle.v1.z),
            Float3::new(triangle.v2.x, triangle.v2.y, triangle.v2.z),
            Float3::new(triangle.v3.x, triangle.v3.y, triangle.v3.z),
        ]);
    }
    output
}

// using a struct for this because i need to store the ocl proque and buffers.
pub struct Renderer {
    proque: ProQue,
    tribuf: Buffer<Float3>,
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
            .create_buffer::<Float3>()
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
