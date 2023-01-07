use rand::prelude::*;
use rusty_engine::prelude::*;

const MARBLE_SPEED: f32 = 600.0;
const CAR_SPEED: f32 = 250.0;
#[derive(Debug)]
struct Enemy {
    health: i32,
    smart: bool,
    label: String,
}

struct GameState {
    marble_labels: Vec<String>,
    cars_left: i32,
    spawn_time: Timer,
    score: i32,
    high_score: i32,
    game_over: bool,
    enemies_vector: Vec<Enemy>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            marble_labels: vec!["marble1".into(), "marble2".into(), "marble3".into()],
            cars_left: 25,
            score: 0,
            high_score: 0,
            spawn_time: Timer::from_seconds(0.0, false),
            game_over: false,
            enemies_vector: Vec::new(),
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
        if label.starts_with("marble") {
            engine.sprites.remove(&label);
            game_state.marble_labels.push(label);
        } else if label.starts_with("car") {
            engine.sprites.remove(&label);
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

            game_state.enemies_vector.push(Enemy {
                preset: car_sprite,
                health: car_health,
                smart: false,
                label: car_label.clone(),
            });

            let car_to_spawn = engine.add_sprite(car_label.clone(), car_sprite);
            car_to_spawn.translation.x = -740.0;
            car_to_spawn.translation.y = thread_rng().gen_range(-100.0..325.0);
            car_to_spawn.collision = true;
        }
    }

    engine
        .sprites
        .values_mut()
        .filter(|sprite| sprite.label.starts_with("car"))
        .for_each(|car| car.translation.x += CAR_SPEED * engine.delta_f32);

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
                let enemy = game_state
                    .enemies_vector
                    .iter_mut()
                    .find(|e| e.label == label)
                    .unwrap();
                enemy.health -= 1;
                if enemy.health == 0 {
        game_state.score += 1;
        let score_t = engine.texts.get_mut("score").unwrap();
        score_t.value = format!("Score: {}", game_state.score);
                    // Handle high score
        if game_state.score > game_state.high_score {
            game_state.high_score = game_state.score;
            let high_score_t = engine.texts.get_mut("high_score").unwrap();
            high_score_t.value = format!("High Score: {}", game_state.high_score);
        }
            engine.sprites.remove(&label);
                    let index = game_state
                        .enemies_vector
                        .iter()
                        .position(|e| e.label == label)
                        .unwrap();
                    game_state.enemies_vector.remove(index);
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
        let score_t = engine.texts.get_mut("score").unwrap();
        score_t.value = format!("Score: {}", game_state.score);
        engine.texts.remove("game_over_text").unwrap();
    }
}
