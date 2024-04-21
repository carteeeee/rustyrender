pub mod engine;
pub mod parser;
pub mod types;

// behold, the shitty cargo tests i made.
#[cfg(test)]
mod tests {
    use super::engine::*;
    use super::types::*;
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

        let geometry = vec![triangle1, triangle2];

        let mut camera = Camera {
            pos: Vec3f::new(0.0, 0.0, 5.0),
            rot: Vec3f::new(0.0, 0.0, 0.0),
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
            let mouse_state = event_pump.mouse_state();

            camera.rot.x = (mouse_state.y() - height as i32 / 2) as f32 / 10.;
            camera.rot.y = (mouse_state.x() - width as i32 / 2) as f32 / 10.;

            if mouse_state.left() {
                camera.pos.z += 0.5;
            }
            if mouse_state.right() {
                camera.pos.z -= 0.5;
            }

            //let now = Instant::now();
            renderer.render(&mut window, &event_pump, &geometry, &camera)?;
            //let elapsed = now.elapsed();
            //println!("Elapsed: {:.2?}", elapsed);
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }

        Ok(()) // wahoo!!
    }
}
