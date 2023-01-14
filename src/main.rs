use std::f32::consts::PI;

use rand::prelude::*;
use rusty_engine::prelude::*;

const MARBLE_SPEED: f32 = 600.0;
const POWERUP_SPEED: f32 = 300.0;

#[derive(Debug)]
struct Enemy {
    health: i32,
    smart: bool,
    label: String,
    speed: f32,
}

struct GameState {
    marble_labels: Vec<String>,
    cars_left: i32,
    spawn_time: Timer,
    explosion_timer: Timer,
    score: i32,
    high_score: i32,
    game_over: bool,
    enemies_vector: Vec<Enemy>,
    power_spawned: bool,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            marble_labels: vec!["marble1".into(), "marble2".into(), "marble3".into()],
            cars_left: 25,
            score: 0,
            high_score: 0,
            spawn_time: Timer::from_seconds(0.0, false),
            explosion_timer: Timer::from_seconds(thread_rng().gen_range(5.0..7.0), true),
            game_over: false,
            enemies_vector: Vec::new(),
            power_spawned: false,
        }
    }
}

impl GameState {
    fn get_enemy(&mut self, label: &String) -> &mut Enemy {
        return self
            .enemies_vector
            .iter_mut()
            .find(|e| e.label == *label)
            .unwrap();
    }

    fn get_enemy_index(&mut self, label: &String) -> usize {
        return self
            .enemies_vector
            .iter()
            .position(|e| e.label == *label)
            .unwrap();
    }

    fn increment_score(&mut self, score_t: &mut Text, high_score_t: &mut Text) {
        self.score += 1;
        score_t.value = format!("Score: {}", self.score);
        if self.score > self.high_score {
            self.high_score = self.score;
            high_score_t.value = format!("High Score: {}", self.high_score);
        }
    }
}

fn main() {
    let mut game = Game::new();

    // Window Settings
    game.window_settings(WindowDescriptor {
        title: "Car Shoot".to_string(),
        ..Default::default()
    });

    // Music
    game.audio_manager.play_music(MusicPreset::Classy8Bit, 0.1);

    let barrel = game.add_sprite("player", SpritePreset::RacingBarrierRed);
    barrel.rotation = UP;
    barrel.scale = 0.5;
    barrel.translation.y = -325.0;
    barrel.layer = 10.0;

    // Text
    let cars_left_t = game.add_text("cars_left", "Cars left: 0");
    cars_left_t.translation = Vec2::new(540.0, -320.0);

    let score_t = game.add_text("score", "Score: 0");
    score_t.translation = Vec2::new(540.0, -280.0);

    let high_score_t = game.add_text("high_score", "High Score: 0");
    high_score_t.translation = Vec2::new(540.0, -240.0);

    game.add_logic(game_logic);
    game.run(GameState::default());
}

fn game_logic(engine: &mut Engine, game_state: &mut GameState) {
    //Set Cars left text
    let cars_left_t = engine.texts.get_mut("cars_left").unwrap();
    cars_left_t.value = format!("Cars left: {}", game_state.cars_left);

    // Handle barrel location
    let player = engine.sprites.get_mut("player").unwrap();
    if let Some(mouse_location) = engine.mouse_state.location() {
        player.translation.x = mouse_location.x;
    }
    let player_x = player.translation.x;

    // Handle marbles
    if engine.mouse_state.just_pressed(MouseButton::Left) {
        if let Some(marble_lable) = game_state.marble_labels.pop() {
            let marble = engine.add_sprite(marble_lable, SpritePreset::RollingBallBlue);
            marble.translation.x = player_x;
            marble.translation.y = -275.0;
            marble.layer = 5.0;
            marble.collision = true;
            engine.audio_manager.play_sfx(SfxPreset::Impact2, 0.4);
        }
    }

    // Move the marbles upwards
    engine
        .sprites
        .values_mut()
        .filter(|sprite| sprite.label.starts_with("marble"))
        .for_each(|marble| marble.translation.y += MARBLE_SPEED * engine.delta_f32);

    let mut labels_to_delete: Vec<String> = Vec::new();
    for (label, sprite) in engine.sprites.iter() {
        if sprite.translation.y > 400.0 || sprite.translation.x > 750.0 {
            labels_to_delete.push(label.clone());
        }
    }
    for label in labels_to_delete {
        engine.sprites.remove(&label);
        if label.starts_with("marble") {
            game_state.marble_labels.push(label);
        } else if label.starts_with("car") {
            let index = game_state
                .enemies_vector
                .iter()
                .position(|e| e.label == label)
                .unwrap();
            game_state.enemies_vector.remove(index);
        }
    }

    // Spawn enenmies
    if game_state.spawn_time.tick(engine.delta).just_finished() {
        game_state.spawn_time = Timer::from_seconds(thread_rng().gen_range(0.1..1.25), false);
        if game_state.cars_left > 0 {
            game_state.cars_left -= 1;
            let cars_left_t = engine.texts.get_mut("cars_left").unwrap();
            cars_left_t.value = format!("Cars left: {}", game_state.cars_left);

            let car_label = format!("car{}", game_state.cars_left);
            let cars_to_choose = vec![
                SpritePreset::RacingCarBlack,
                SpritePreset::RacingCarRed,
                SpritePreset::RacingCarBlue,
                SpritePreset::RacingCarGreen,
                SpritePreset::RacingCarYellow,
            ];

            let car_sprite = *cars_to_choose.iter().choose(&mut thread_rng()).unwrap();

            let car_health = match car_sprite {
                SpritePreset::RacingCarGreen => 2,
                _ => 1,
            };

            let car_behaviour = match car_sprite {
                SpritePreset::RacingCarBlack => true,
                _ => false,
            };

            let car_speed: f32 = match car_sprite {
                SpritePreset::RacingCarBlack => 150.0,
                SpritePreset::RacingCarBlue => 200.0,
                SpritePreset::RacingCarGreen => 250.0,
                SpritePreset::RacingCarYellow => 300.0,
                SpritePreset::RacingCarRed => 350.0,
                _ => 0.0,
            };

            game_state.enemies_vector.push(Enemy {
                health: car_health,
                smart: car_behaviour,
                label: car_label.clone(),
                speed: car_speed,
            });

            let car_to_spawn = engine.add_sprite(car_label, car_sprite);
            car_to_spawn.translation.x = -740.0;
            car_to_spawn.translation.y = thread_rng().gen_range(-100.0..325.0);
            car_to_spawn.collision = true;
        }
    }

    engine
        .sprites
        .values_mut()
        .filter(|sprite| sprite.label.starts_with("car"))
        .for_each(|car| {
            let car_speed = game_state.get_enemy(&car.label).speed;
            car.translation.x += car_speed * engine.delta_f32;
            if game_state.get_enemy(&car.label).smart {
                car.translation.y +=
                    5.0 * (2.0 * PI * 0.5 * engine.time_since_startup_f64 as f32).sin();
            }
        });

    // Spawn powerups
    if game_state
        .explosion_timer
        .tick(engine.delta)
        .just_finished()
        && game_state.power_spawned == false
    {
        let explosion_s = engine.add_sprite("power_explosion", "sprite/racing/explosion.png");
        explosion_s.translation.x = -740.0;
        explosion_s.translation.y = thread_rng().gen_range(-100.0..325.0);
        explosion_s.collision = true;
        explosion_s.scale = 0.5;
        game_state.power_spawned = true;
    }

    // Move PowerUps
    engine
        .sprites
        .values_mut()
        .filter(|sprite| sprite.label.starts_with("power"))
        .for_each(|s| s.translation.x += POWERUP_SPEED * engine.delta_f32);

    for colision in engine.collision_events.drain(..) {
        if colision.state == CollisionState::End {
            continue;
        }

        if !colision.pair.one_starts_with("marble") {
            continue;
        }

        for label in colision.pair {
            if label.starts_with("marble") {
                engine.sprites.remove(&label);
                game_state.marble_labels.push(label);
            } else if label.starts_with("car") {
                let enemy = game_state.get_enemy(&label);
                enemy.health -= 1;
                if enemy.health == 0 {
                    let [score_t, high_score_t] =
                        engine.texts.get_many_mut(["score", "high_score"]).unwrap();
                    game_state.increment_score(score_t, high_score_t);

                    let index = game_state.get_enemy_index(&label);
                    game_state.enemies_vector.remove(index);
                    engine.sprites.remove(&label);
                }
            } else if label.starts_with("power_explosion") {
                engine.sprites.remove(&label);
                engine.audio_manager.play_sfx("sfx/explosion.mp3", 0.5);
                let mut labels_to_delete: Vec<String> = Vec::new();
                for (label, sprite) in engine.sprites.iter() {
                    if sprite.label.starts_with("car") {
                        labels_to_delete.push(label.to_string());
                        let index = game_state.get_enemy_index(&label);
                        game_state.enemies_vector.remove(index);
                    }
        }

                for label in labels_to_delete {
                    let [score_t, high_score_t] =
                        engine.texts.get_many_mut(["score", "high_score"]).unwrap();
                    game_state.increment_score(score_t, high_score_t);
            engine.sprites.remove(&label);
                }
            }
            engine.audio_manager.play_sfx(SfxPreset::Confirmation1, 0.2);
        }
    }

    // Check end of game. The game ends when there are no car sprites and game_state.cars_left == 0
    let mut car_sprites = engine
        .sprites
        .values()
        .filter(|sprite| sprite.label.starts_with("car"))
        .peekable();

    if car_sprites.next().is_none() && game_state.cars_left == 0 && !game_state.game_over {
        game_state.game_over = true;
        let game_over_t = engine.add_text("game_over_text", "Game Over! \nPress R to play again");
        game_over_t.translation = Vec2::new(0.0, 0.0);
        engine.audio_manager.play_sfx(SfxPreset::Jingle2, 0.2);
    }

    // Start the game again
    if engine.keyboard_state.just_pressed(KeyCode::R) && game_state.game_over {
        game_state.game_over = false;
        game_state.cars_left = 25;
        game_state.score = 0;
        game_state.power_spawned = false;
        let score_t = engine.texts.get_mut("score").unwrap();
        score_t.value = format!("Score: {}", game_state.score);
        engine.texts.remove("game_over_text").unwrap();
    }
}
