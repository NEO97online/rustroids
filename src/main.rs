#![feature(drain_filter)]

mod canvas;
mod util;

extern crate rand;
extern crate minifb;
extern crate bitfont;

use minifb::{Key, KeyRepeat, Scale, ScaleMode, Window, WindowOptions};

use std::time::{Duration, Instant};

use crate::util::wrap;
use crate::canvas::Canvas;

const WIDTH: usize = 320;
const HEIGHT: usize = 180;

struct SpaceObject {
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
    size: f32,
    angle: f32,
}

fn collide_circle(cx: f32, cy: f32, radius: f32, x: f32, y: f32) -> bool {
    (x - cx).powi(2) + (y - cy).powi(2) < radius.powi(2)
}

fn main() {
    let mut canvas = Canvas::new(WIDTH, HEIGHT);

    let mut window = Window::new(
        "Rustroids - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions {
            borderless: false,
            title: true,
            resize: false,
            scale: Scale::X2,
            scale_mode: ScaleMode::AspectRatioStretch,
            topmost: false,
            transparency: false,
            none: false,
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });
    
    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(Duration::from_micros(16600)));
    
    // initialize game state
    let mut asteroids = vec![
        SpaceObject { x: 20.0, y: 20.0, dx: 16.0, dy: -20.0, size: 16.0, angle: 0.0 },
        SpaceObject { x: 100.0, y: 20.0, dx: -8.0, dy: -13.0, size: 16.0, angle: 0.0 },
    ];
    let mut player = SpaceObject {
        x: (WIDTH / 2) as f32,
        y: (HEIGHT / 2) as f32,
        dx: 0.0,
        dy: 0.0,
        size: 1.0,
        angle: 0.0,
    };
    let mut bullets: Vec<SpaceObject> = Vec::new();
    let mut score = 0;
    
    let mut last_update = Instant::now();

    let asteroid_model = {
        let mut points: Vec<(f32, f32)> = Vec::new();
        let asteroid_verts = 20;
        for i in 0..asteroid_verts {
            let rng: f32 = rand::random();
            let radius: f32 = rng * 0.4 + 0.8;
            println!("{}", radius);
            let a = (i as f32 / asteroid_verts as f32) * 6.28318;
            let vertex = (radius * a.sin(), radius * a.cos());
            points.push(vertex);
        }
        points
    };
    
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let delta = last_update.elapsed().as_secs_f32();
        last_update = Instant::now();
        
        canvas.clear(0x000000);
        
        // player input
        if window.is_key_down(Key::W) || window.is_key_down(Key::K) {
            player.dx += player.angle.sin() * 100.0 * delta;
            player.dy += -player.angle.cos() * 100.0 * delta;
        }
        if window.is_key_down(Key::A) || window.is_key_down(Key::H) {
            player.angle -= 5.0 * delta;
        }
        if window.is_key_down(Key::D) || window.is_key_down(Key::L) {
            player.angle += 5.0 * delta;
        }
        if window.is_key_pressed(Key::Space, KeyRepeat::No) {
            bullets.push(SpaceObject {
                x: player.x,
                y: player.y,
                dx: 100.0 * player.angle.sin(),
                dy: -100.0 * player.angle.cos(),
                angle: 0.0,
                size: 1.0,
            })
        }
        
        // update & draw asteroids
        for asteroid in asteroids.iter_mut() {
            asteroid.x += asteroid.dx * delta;
            asteroid.y += asteroid.dy * delta;
            
            asteroid.x = wrap(asteroid.x, WIDTH);
            asteroid.y = wrap(asteroid.y, HEIGHT);
            
            asteroid.angle += 0.5 * delta;
            
            canvas.draw_wireframe_model(
                &asteroid_model,
                asteroid.x,
                asteroid.y,
                asteroid.angle, 
                asteroid.size,
                0xffff00,
            );
        }
        
        // update player
        player.x += player.dx * delta;
        player.y += player.dy * delta;

        player.x = wrap(player.x, WIDTH);
        player.y = wrap(player.y, HEIGHT);
        
        // player vertices
        let player_model = vec![
            (0.0, -5.0),
            (-2.5, 2.5),
            (2.5, 2.5),
        ];
        
        // draw player
        canvas.draw_wireframe_model(
            &player_model,
            player.x,
            player.y,
            player.angle,
            player.size,
            0xffffff
        );
        
        // update, draw, and cull bullets
        bullets.drain_filter(|b| {
            b.x += b.dx * delta;
            b.y += b.dy * delta;

            canvas.draw(b.x, b.y, 0xffffff);
            
            let mut new_asteroids: Vec<SpaceObject> = Vec::new();
            let mut has_collision = false;
            
            // check collision with asteroids
            asteroids.drain_filter(|a| {
                if collide_circle(a.x, a.y, a.size, b.x, b.y) {
                    has_collision = true;
                    score += 25;
                    // spawn child asteroids    
                    if a.size > 4.0 {
                        for _ in 0..2 {
                            let rng: f32 = rand::random();
                            let angle = rng * 6.283185;
                            new_asteroids.push(SpaceObject {
                                x: a.x,
                                y: a.y,
                                dx: 10.0 * angle.sin(),
                                dy: 10.0 * angle.cos(),
                                size: a.size / 2.0,
                                angle,
                            })
                        }
                    }
                    true
                } else {
                    false
                }
            });
            // append newly created asteroids
            asteroids.extend(new_asteroids);
            
            // returns true if bullet is off-screen or collided, removing it from the vector
            has_collision || b.x < 0.0 || b.y < 0.0 || b.x > WIDTH as f32 || b.y > HEIGHT as f32
        });
        
        // draw GUI
        canvas.draw_text(format!("Score: {}", score).as_str(), 5.0, 5.0, 0x448aff); 
        
        window
            .update_with_buffer(&canvas.buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
