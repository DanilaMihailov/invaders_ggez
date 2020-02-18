use ggez::audio::SoundSource;
use ggez::input::keyboard::KeyCode;
use ggez::input::keyboard::KeyMods;
use ggez::*;
use rand;
use rand::Rng;

const SCREEN_SIZE: (f32, f32) = (800.0, 600.0);
const PLAYER_HEIGHT: f32 = 100.0;

#[derive(Clone, Copy, Debug)]
struct Point {
    x: f32,
    y: f32,
}

impl From<Point> for mint::Point2<f32> {
    fn from(p: Point) -> Self {
        mint::Point2::<f32> {
            x: p.x,
            y: SCREEN_SIZE.1 - p.y,
        }
    }
}

struct State {
    input: InputState,
    player: Player,
    bullets: Vec<Bullet>,
    invaders: Vec<Invader>,
    points: u64,
    invader_image: graphics::Image,
    bg_image: graphics::Image,
    font: graphics::Font,
    shot_sound: audio::Source,
    hit_sound: audio::Source,
    bullet_image: graphics::Image,
}

impl State {
    fn new(ctx: &mut Context) -> Self {
        let shot_sound = audio::Source::new(ctx, "/pew.ogg").unwrap();
        let hit_sound = audio::Source::new(ctx, "/boom.ogg").unwrap();

        State {
            input: Default::default(),
            player: Player::new(ctx),
            bullets: Vec::new(),
            invaders: Vec::new(),
            points: 0,
            invader_image: graphics::Image::new(ctx, "/invader.png").unwrap(),
            bullet_image: graphics::Image::new(ctx, "/bullet.png").unwrap(),
            bg_image: graphics::Image::new(ctx, "/bg.jpg").unwrap(),
            font: graphics::Font::new(ctx, "/DejaVuSerif.ttf").unwrap(),
            shot_sound,
            hit_sound,
        }
    }

    fn restart(&mut self) {
        self.input = Default::default();
        self.bullets = Vec::new();
        self.invaders = Vec::new();
        self.points = 0;
        self.player.reset();
    }
}

#[derive(Debug)]
struct InputState {
    xaxis: f32,
    yaxis: f32,
    fire: bool,
}

impl Default for InputState {
    fn default() -> Self {
        InputState {
            xaxis: 0.0,
            yaxis: 0.0,
            fire: false,
        }
    }
}

#[derive(Debug)]
struct Bullet {
    position: Point,
    size: (f32, f32),
}

impl Bullet {
    fn new(point: Point) -> Self {
        Bullet {
            position: Point {
                x: point.x + 40.0,
                ..point
            },
            size: (20.0, 20.0),
        }
    }

    fn draw(&self, ctx: &mut Context, img: &graphics::Image) -> GameResult {
        let point: mint::Point2<f32> = self.position.into();
        graphics::draw(
            ctx,
            img,
            graphics::DrawParam::new()
                .dest(point)
                .rotation(1.570796)
                .scale([0.06, 0.06])
                .offset([0.0, 0.9]),
        )?;

        // target rect
        // let rect = graphics::Mesh::new_rectangle(
        //     ctx,
        //     graphics::DrawMode::stroke(2.0),
        //     self.get_rect(),
        //     graphics::Color::from_rgb(255, 0, 0),
        // )?;
        // graphics::draw(ctx, &rect, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;

        Ok(())
    }
}

impl Entity for Bullet {
    fn get_rect(&self) -> graphics::Rect {
        let point: mint::Point2<f32> = self.position.into();
        graphics::Rect::new(point.x, point.y, self.size.0, self.size.1)
    }
}

trait Entity {
    fn get_rect(&self) -> graphics::Rect;
}

enum MoveState {
    Forward,
    Backwards,
}

struct Moves {
    allowed: (f32, f32),
    current: (f32, f32),
    state: MoveState,
}

impl Moves {
    fn new() -> Self {
        let mut rng = rand::thread_rng();

        Self {
            allowed: (rng.gen_range(50.0, 300.0), 0.0),
            current: (0.0, 0.0),
            state: MoveState::Forward,
        }
    }
}

struct Invader {
    position: Point,
    size: (f32, f32),
    health: u8,
    movement: Moves,
}

impl Invader {
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        Invader {
            position: Point {
                x: rng.gen_range(100.0, SCREEN_SIZE.0 - 100.0),
                y: rng.gen_range(SCREEN_SIZE.1 / 2.0, SCREEN_SIZE.1 - 100.0),
            },
            size: (80.0, 60.0),
            health: 1,
            movement: Moves::new(),
        }
    }

    fn draw(&self, ctx: &mut Context, img: &graphics::Image) -> GameResult {
        let point: mint::Point2<f32> = self.position.into();
        graphics::draw(
            ctx,
            img,
            graphics::DrawParam::new()
                .dest(point)
                .scale([0.1, 0.1])
                .offset([0.13, 0.23]),
        )?;

        // target rect
        // let rect = graphics::Mesh::new_rectangle(
        //     ctx,
        //     graphics::DrawMode::stroke(2.0),
        //     self.get_rect(),
        //     graphics::Color::from_rgb(255, 0, 0),
        // )?;
        // graphics::draw(ctx, &rect, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;

        Ok(())
    }
}

impl Entity for Invader {
    fn get_rect(&self) -> graphics::Rect {
        let point: mint::Point2<f32> = self.position.into();
        graphics::Rect::new(point.x, point.y, self.size.0, self.size.1)
    }
}

struct Player {
    position: Point,
    image: graphics::Image,
    health: u8,
    shot_timeout: f32,
}

impl Entity for Player {
    fn get_rect(&self) -> graphics::Rect {
        let point: mint::Point2<f32> = self.position.into();
        graphics::Rect::new(point.x, point.y, 95.0, PLAYER_HEIGHT)
    }
}

impl Player {
    fn new(ctx: &mut Context) -> Self {
        Player {
            position: Point {
                x: SCREEN_SIZE.0 / 2.0,
                y: PLAYER_HEIGHT,
            },
            image: graphics::Image::new(ctx, "/player.png").unwrap(),
            health: 3,
            shot_timeout: 0.0,
        }
    }
    fn reset(&mut self) {
        self.position = Point {
            x: SCREEN_SIZE.0 / 2.0,
            y: PLAYER_HEIGHT,
        };
        self.health = 3;
        self.shot_timeout = 0.0;
    }

    fn draw(&self, ctx: &mut Context) -> GameResult {
        let point: mint::Point2<f32> = self.position.into();
        graphics::draw(
            ctx,
            &self.image,
            graphics::DrawParam::new()
                .dest(point)
                .scale([0.2, 0.2])
                .offset([0.05, 0.0]),
        )?;
        // target rect
        // let rect = graphics::Mesh::new_rectangle(
        //     ctx,
        //     graphics::DrawMode::stroke(2.0),
        //     self.get_rect(),
        //     graphics::Color::from_rgb(255, 0, 0),
        // )?;
        // graphics::draw(ctx, &rect, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;
        Ok(())
    }
}

const SHOT_TIMEOUT: f32 = 0.3;
const INVADER_SPEED: f32 = 3.0;

impl event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if self.player.health == 0 {
            return Ok(());
        }
        while timer::check_update_time(ctx, 60) {
            let seconds = 1.0 / 60f32;

            if self.input.xaxis > 0.0
                && self.player.position.x < (SCREEN_SIZE.0 - self.player.get_rect().w)
            {
                self.player.position.x += self.input.xaxis;
            }

            if self.input.xaxis < 0.0 && self.player.position.x > 0.0 {
                self.player.position.x += self.input.xaxis;
            }

            if self.input.yaxis > 0.0 && self.player.position.y < SCREEN_SIZE.1 {
                self.player.position.y += self.input.yaxis;
            }

            if self.input.yaxis < 0.0 && self.player.position.y > self.player.get_rect().h {
                self.player.position.y += self.input.yaxis;
            }

            self.player.shot_timeout -= seconds;

            if self.input.fire && self.player.shot_timeout < 0.0 {
                self.bullets.push(Bullet::new(self.player.position));
                self.shot_sound.play()?;
                self.player.shot_timeout = SHOT_TIMEOUT;
            }

            while self.invaders.len() < 5 {
                self.invaders.push(Invader::new())
            }

            for bullet in &mut self.bullets {
                bullet.position.y += 20.0;
                let br = bullet.get_rect();

                for invader in &mut self.invaders {
                    if invader.health > 0 {
                        if invader.get_rect().overlaps(&br) {
                            self.hit_sound.play()?;
                            invader.health -= 1;

                            if invader.health == 0 {
                                self.points += 1;
                            }

                            bullet.position.y = SCREEN_SIZE.1;
                            break;
                        }
                    }
                }
            }

            self.bullets.retain(|s| s.position.y < SCREEN_SIZE.1);
            self.invaders.retain(|i| i.health > 0 && i.position.y > 0.0);

            let pl_rect = self.player.get_rect();
            for invader in &mut self.invaders {
                invader.position.y -= 1.0;
                if invader.position.y <= 0.0 {
                    self.points -= 1;
                }
                match invader.movement.state {
                    MoveState::Forward => {
                        invader.position.x += INVADER_SPEED;
                        invader.movement.current.0 += INVADER_SPEED;
                        if invader.movement.allowed.0 < invader.movement.current.0
                            || invader.position.x > (SCREEN_SIZE.0 - invader.get_rect().w)
                        {
                            invader.movement.state = MoveState::Backwards;
                        }
                    }
                    MoveState::Backwards => {
                        invader.position.x -= INVADER_SPEED;
                        invader.movement.current.0 -= INVADER_SPEED;
                        if 0.0 > invader.movement.current.0 || invader.position.x < 0.0 {
                            invader.movement.state = MoveState::Forward;
                        }
                    }
                }
                if invader.health > 0 && invader.get_rect().overlaps(&pl_rect) {
                    invader.health = 0;
                    if self.player.health > 0 {
                        self.player.health -= 1;
                        self.hit_sound.play()?;
                    }
                }
            }
        }

        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::BLACK);

        if self.player.health > 0 {
            graphics::draw(
                ctx,
                &self.bg_image,
                graphics::DrawParam::new()
                    .dest([0.0, 0.0])
                    .scale([0.5, 0.5]), // .offset([0.13, 0.23]),
            )?;
        }

        for bullet in &self.bullets {
            bullet.draw(ctx, &self.bullet_image)?;
        }

        for invader in &self.invaders {
            invader.draw(ctx, &self.invader_image)?;
        }

        self.player.draw(ctx)?;

        // And draw the GUI elements in the right places.
        let hp_dest = [10.0, 10.0];
        let score_dest = [100.0, 10.0];

        let hp_str = format!("HP: {}", self.player.health);
        let score_str = format!("Score: {}", self.points);
        let level_display = graphics::Text::new((hp_str, self.font, 32.0));
        let score_display = graphics::Text::new((score_str, self.font, 32.0));
        graphics::draw(ctx, &level_display, (hp_dest, 0.0, graphics::WHITE))?;
        graphics::draw(ctx, &score_display, (score_dest, 0.0, graphics::WHITE))?;

        if self.player.health == 0 {
            let go_text = graphics::Text::new(("GAME OVER", self.font, 100.0));
            graphics::draw(
                ctx,
                &go_text,
                (
                    [100.0, (SCREEN_SIZE.1 / 2.0) - 50.0],
                    0.0,
                    graphics::Color::from_rgb(255, 0, 0),
                ),
            )?;
        } else {
        }

        graphics::present(ctx)?;
        timer::yield_now();
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        key_code: KeyCode,
        _key_mods: KeyMods,
        _: bool,
    ) {
        match key_code {
            KeyCode::Right => self.input.xaxis = 10.0,
            KeyCode::Left => self.input.xaxis = -10.0,
            KeyCode::Up => self.input.yaxis = 10.0,
            KeyCode::Down => self.input.yaxis = -10.0,
            KeyCode::Space => self.input.fire = true,
            KeyCode::Escape => ggez::event::quit(ctx),
            _ => (),
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, key_code: KeyCode, _key_mods: KeyMods) {
        match key_code {
            KeyCode::Space => self.input.fire = false,
            KeyCode::Right => {
                if self.input.xaxis > 0.0 {
                    self.input.xaxis = 0.0
                }
            }
            KeyCode::Left => {
                if self.input.xaxis < 0.0 {
                    self.input.xaxis = 0.0
                }
            }
            KeyCode::Up => {
                if self.input.yaxis > 0.0 {
                    self.input.yaxis = 0.0
                }
            }
            KeyCode::Down => {
                if self.input.yaxis < 0.0 {
                    self.input.yaxis = 0.0
                }
            }
            KeyCode::Return => {
                self.restart();
            }
            _ => (),
        }
    }
}

fn main() -> GameResult {
    use std::path;
    let (ctx, event_loop) = &mut ggez::ContextBuilder::new("plat", "dan")
        .add_resource_path(path::PathBuf::from("./resources"))
        .window_setup(conf::WindowSetup::default().title("Plat!"))
        .window_mode(conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        .build()?;
    let mut state: State = State::new(ctx);
    event::run(ctx, event_loop, &mut state)
}
