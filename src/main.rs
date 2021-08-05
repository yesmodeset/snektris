use std::time::{Duration, Instant};
use tetra::graphics::{self, Color, Texture};
use tetra::input::{self, Key};
use tetra::{Context, ContextBuilder, State};
use tetra::math::Vec2;
use array2d::Array2D;
use fastrand;

const WINDOW_WIDTH:  f32 = 480.0;
const WINDOW_HEIGHT: f32 = 480.0;
const MAP_WIDTH:  usize = 9;
const MAP_HEIGHT: usize = 9;
const SPEED: u64 = 333;

fn main() -> tetra::Result {
    ContextBuilder::new("stupid fuck_v2", WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
        .multisampling(8)
        .quit_on_escape(true)
        .build()?
        .run(GameState::new)
}

#[derive(PartialEq, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right
}

struct Snake {
    body: Vec<Vec2<usize>>,
    length: usize,
    last_move: Instant,
    direction_queue: Vec<Direction>,
    disabled: bool,
}

impl Snake {
    fn new() -> Snake {
        let length = 3;
        let body = vec![Vec2::new(3, 1); length];
        Snake {
            body,
            length,
            last_move: Instant::now(),
            direction_queue: vec!(Direction::Right),
            disabled: false,
        }
    }

    fn update(&mut self) {
        let head: Vec2<i32> = self.body.first().unwrap().as_();
        let new = match &self.direction_queue.first().unwrap() {
            Direction::Up    => head + Vec2::new(0, -1),
            Direction::Down  => head + Vec2::new(0, 1),
            Direction::Left  => head + Vec2::new(-1, 0),
            Direction::Right => head + Vec2::new(1, 0),
        };
        self.body.insert(0, new.as_());
        if self.body.len() > self.length {
            self.body.pop();
        }
    }
}

#[derive(PartialEq)]
struct FallingBlocks {
    body: Vec<Vec2<usize>>,
    length: usize,
    last_move: Instant,
    hit_ground: bool,
}

impl FallingBlocks {
    fn from_snake(snake: &Snake) -> FallingBlocks {
        FallingBlocks {
            length: snake.length,
            body: snake.body.clone(),
            last_move: Instant::now(),
            hit_ground: false,
        }
    }

    fn update(&mut self, map: &Array2D<bool>) {
        for part in self.body.iter() {
            if part.y >= MAP_HEIGHT - 1 || map[(part.x, part.y + 1)] {
                self.hit_ground = true;
                return;
            }
        }
        for part in self.body.iter_mut() {
            part.y += 1;
        }
    }
}

#[derive(PartialEq)]
struct Fruit {
    pos: Vec2<usize>,
}

impl Fruit {
    fn new(pos: Vec2<usize>) -> Fruit {
        Fruit {
            pos
        }
    }
}

enum PlayState {
    Normal,
    Falling,
}

struct GameState {
    snake: Snake,
    map: Array2D<bool>,
    fruits: Vec<Fruit>,
    falling_blocks: Option<FallingBlocks>,
    texture: Texture,
    state: PlayState,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        let map = Array2D::filled_with(
            true, MAP_WIDTH, MAP_HEIGHT);
        let fruits = vec!(Fruit::new(Vec2::new(fastrand::usize(0..(MAP_WIDTH as usize)), 
                                               fastrand::usize(0..(MAP_HEIGHT as usize)))));
        let texture = Texture::new(ctx, "./snake.png")?;

        Ok(GameState {
            snake: Snake::new(),
            map,
            fruits,
            falling_blocks: None,
            texture,
            state: PlayState::Normal,
        })
    }

    fn blocks_to_map(&mut self) {
        if let Some(falling_blocks) = &self.falling_blocks {
            for block in falling_blocks.body.iter() {
                self.map[(block.x, block.y)] = true;
            }
        }
    }
}

impl State for GameState {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.5, 0.5, 1.0));
        
        let part_size = f32::min(WINDOW_WIDTH, WINDOW_HEIGHT) / 
                        f32::min(MAP_WIDTH as f32, MAP_HEIGHT as f32);
        let scale = Vec2::new(part_size / self.texture.width()  as f32,
                                     part_size / self.texture.height() as f32);

        if !self.snake.disabled {
            for part in self.snake.body.iter() {
                let params = graphics::DrawParams::new()
                    .position(Vec2::new(part.x as f32 * part_size, part.y as f32 * part_size))
                    .scale(scale)
                    .color(graphics::Color::rgb(0.5, 1.0, 0.5));
                self.texture.draw(ctx, params);
            }
        }

        if let Some(falling_blocks) = &self.falling_blocks {
            for part in falling_blocks.body.iter() {
                let params = graphics::DrawParams::new()
                    .position(Vec2::new(part.x as f32 * part_size, part.y as f32 * part_size))
                    .scale(scale)
                    .color(graphics::Color::rgb(0.5, 0.5, 0.5));
                self.texture.draw(ctx, params); 
            }
        }


        for fruit in self.fruits.iter() {
            let params = graphics::DrawParams::new()
                .position(Vec2::new(fruit.pos.x as f32 * part_size, fruit.pos.y as f32 * part_size))
                .scale(scale)
                .color(graphics::Color::rgb(1.0, 0.5, 0.5));
            self.texture.draw(ctx, params); 
        }


        for y in 0..(MAP_HEIGHT) {
            for x in 0..(MAP_WIDTH) {
                if self.map[(x, y)] {
                    let coords = Vec2::new(x, y);
                    let params = graphics::DrawParams::new()
                        .position(Vec2::new(coords.x as f32 * part_size, coords.y as f32 * part_size))
                        .scale(scale)
                        .color(graphics::Color::rgb(0.5, 0.5, 0.5));
                    self.texture.draw(ctx, params); 
                }
            }
        }

        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        match self.state {
            PlayState::Normal => {
                let mut to_remove = Vec::new();
                for y in 0..(MAP_HEIGHT) {
                    let mut to_to_remove = true;
                    for x in 0..(MAP_WIDTH) {
                        if !self.map[(x, y)] {
                            to_to_remove = false;
                        }
                    }
                    if to_to_remove {
                        to_remove.push(y);
                    }
                }
                for y in to_remove.iter() {
                    for x in 0..(MAP_WIDTH) {
                        self.map[(x, *y)] = false;
                    }
                    for yy in (0..(*y)).rev() {
                        for x in 0..(MAP_WIDTH) {
                            self.map[(x, yy + 1)] = self.map[(x, yy)];
                        }
                    }
                }

                if !self.snake.disabled {
                    let mut fruits_to_retain = Vec::new();
                    let head = *self.snake.body.first().unwrap();
                    for fruit in self.fruits.iter_mut() {
                        if fruit.pos == head {
                            self.falling_blocks = Some(FallingBlocks::from_snake(&self.snake));
                            self.snake.disabled = true;
                            fruits_to_retain.push(false);
                            self.state = PlayState::Falling;
                        } else {
                            fruits_to_retain.push(true);
                        }
                    }

                    let mut count = 0;
                    for bodypart in 1..self.snake.body.len() {
                        if head == self.snake.body[bodypart] {
                            count += 1;
                        }
                    }
                    //ugly
                    let coords = head.into_tuple();
                    if head.x > MAP_WIDTH  || head.x < 0 || 
                       head.y > MAP_HEIGHT || head.y < 0 || 
                       count == 1 || self.map[coords] {
                        println!("{}", head);
                        panic!();
                    }

                    for key in input::get_keys_pressed(ctx) {
                        let current_direction = self.snake.direction_queue.last().unwrap();
                        let next_direction = match key {
                            Key::Up    => Direction::Up,
                            Key::Down  => Direction::Down,
                            Key::Left  => Direction::Left,
                            Key::Right => Direction::Right,
                            _ => *current_direction,
                        };

                        let forbidden_direction = match current_direction {
                            Direction::Up    => Direction::Down,
                            Direction::Down  => Direction::Up,
                            Direction::Left  => Direction::Right,
                            Direction::Right => Direction::Left,
                        };
                        
                        if next_direction != forbidden_direction &&
                           next_direction != *current_direction {
                            self.snake.direction_queue.push(next_direction);
                        }
                    }

                    if self.snake.last_move.elapsed() >= Duration::from_millis(SPEED) {
                        if self.snake.direction_queue.len() > 1 {
                            self.snake.direction_queue.remove(0);
                        }
                        self.snake.direction_queue.truncate(2);
                        self.snake.update();
                        self.snake.last_move = Instant::now();
                    }
                    let mut iter = fruits_to_retain.iter();
                    self.fruits.retain(|_| *iter.next().unwrap());
                }
            }
            PlayState::Falling => {
                let mut fallen = false;
                if let Some(falling_blocks) = &mut self.falling_blocks {
                    if falling_blocks.hit_ground {
                        fallen = true;
                    }

                    if falling_blocks.last_move.elapsed() >= Duration::from_millis(SPEED / 3) {
                        falling_blocks.update(&self.map);
                        falling_blocks.last_move = Instant::now();
                    }
                }
                if fallen {
                    self.blocks_to_map();
                    self.falling_blocks = None;
                    self.snake.disabled = false;
                    self.snake.length += 1;
                    self.snake.body.fill(Vec2::new(MAP_WIDTH / 2, 0));
                    self.snake.direction_queue[0] = Direction::Down;


                    //ugly
                    let mut coords = (fastrand::usize(0..(MAP_WIDTH)), 
                                  fastrand::usize(0..(MAP_HEIGHT)));
                    while self.map[coords] {
                        coords = (fastrand::usize(0..(MAP_WIDTH)), 
                                  fastrand::usize(0..(MAP_HEIGHT)));
                    }
                    self.fruits.push(Fruit::new(Vec2::from(coords)));
                    self.state = PlayState::Normal;
                }
            }
        }
        Ok(())
    }
}












//use std::time::{Duration, Instant};
//use tetra::graphics::{self, Color, Texture};
//use tetra::input::{self, Key};
//use tetra::{Context, ContextBuilder, State};
//use tetra::math::Vec2;
//use array2d::Array2D;
//use fastrand;
//
//const WINDOW_WIDTH:  f32 = 480.0;
//const WINDOW_HEIGHT: f32 = 480.0;
//const MAP_WIDTH:  usize = 15;
//const MAP_HEIGHT: usize = 15;
//const SPEED: u64 = 333;
//
//fn main() -> tetra::Result {
//    ContextBuilder::new("stupid fuck_v2", WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
//        .multisampling(8)
//        .quit_on_escape(true)
//        .build()?
//        .run(GameState::new)
//}
//
//#[derive(PartialEq, Clone, Copy)]
//enum Direction {
//    Up,
//    Down,
//    Left,
//    Right
//}
//
//struct Snake {
//    body: Vec<Vec2<i32>>,
//    length: usize,
//    last_move: Instant,
//    direction_queue: Vec<Direction>,
//    disabled: bool,
//}
//
//impl Snake {
//    fn new() -> Snake {
//        let length = 3;
//        let body = vec![Vec2::new(3, 1); length];
//        Snake {
//            body,
//            length,
//            last_move: Instant::now(),
//            direction_queue: vec!(Direction::Right),
//            disabled: false,
//        }
//    }
//
//    fn update(&mut self) {
//        let new = match &self.direction_queue.first().unwrap() {
//            Direction::Up    => self.body.first().unwrap() + Vec2::new(0, -1),
//            Direction::Down  => self.body.first().unwrap() + Vec2::new(0, 1),
//            Direction::Left  => self.body.first().unwrap() + Vec2::new(-1, 0),
//            Direction::Right => self.body.first().unwrap() + Vec2::new(1, 0),
//        };
//        self.body.insert(0, new);
//        if self.body.len() > self.length {
//            self.body.pop();
//        }
//    }
//}
//
//#[derive(PartialEq)]
//struct FallingBlocks {
//    body: Vec<Vec2<i32>>,
//    length: usize,
//    last_move: Instant,
//    hit_ground: bool,
//}
//
//impl FallingBlocks {
//    fn from_snake(snake: &Snake) -> FallingBlocks {
//        FallingBlocks {
//            length: snake.length,
//            body: snake.body.clone(),
//            last_move: Instant::now(),
//            hit_ground: false,
//        }
//    }
//
//    fn update(&mut self, map: &Array2D<bool>) {
//        for part in self.body.iter() {
//            let x = part.x as usize;
//            let y = part.y as usize;
//            if y >= MAP_HEIGHT - 1 || map[(x, y + 1)] {
//                self.hit_ground = true;
//                return;
//            }
//        }
//        for part in self.body.iter_mut() {
//            part.y += 1;
//        }
//    }
//}
//
//#[derive(PartialEq)]
//struct Fruit {
//    pos: Vec2<i32>,
//}
//
//impl Fruit {
//    fn new(pos: Vec2<i32>) -> Fruit {
//        Fruit {
//            pos
//        }
//    }
//}
//
//enum PlayState {
//    Normal,
//    Falling,
//}
//
//struct GameState {
//    snake: Snake,
//    map: Array2D<bool>,
//    fruits: Vec<Fruit>,
//    falling_blocks: Option<FallingBlocks>,
//    texture: Texture,
//    state: PlayState,
//}
//
//impl GameState {
//    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
//        let map = Array2D::filled_with(
//            true, MAP_WIDTH, MAP_HEIGHT);
//        let fruits = vec!(Fruit::new(Vec2::new(fastrand::i32(0..(MAP_WIDTH as i32)), 
//                                               fastrand::i32(0..(MAP_HEIGHT as i32)))));
//        let texture = Texture::new(ctx, "./snake.png")?;
//
//        Ok(GameState {
//            snake: Snake::new(),
//            map,
//            fruits,
//            falling_blocks: None,
//            texture,
//            state: PlayState::Normal,
//        })
//    }
//
//    fn blocks_to_map(&mut self) {
//        if let Some(falling_blocks) = &self.falling_blocks {
//            for block in falling_blocks.body.iter() {
//                self.map[(block.x as usize, block.y as usize)] = true;
//            }
//        }
//    }
//}
//
//impl State for GameState {
//    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
//        graphics::clear(ctx, Color::rgb(0.5, 0.5, 1.0));
//        
//        let part_size = f32::min(WINDOW_WIDTH, WINDOW_HEIGHT) / 
//                        f32::min(MAP_WIDTH as f32, MAP_HEIGHT as f32);
//        let scale = Vec2::new(part_size / self.texture.width()  as f32,
//                                     part_size / self.texture.height() as f32);
//
//        if !self.snake.disabled {
//            for part in self.snake.body.iter() {
//                let params = graphics::DrawParams::new()
//                    .position(Vec2::new(part.x as f32 * part_size, part.y as f32 * part_size))
//                    .scale(scale)
//                    .color(graphics::Color::rgb(0.5, 1.0, 0.5));
//                self.texture.draw(ctx, params);
//            }
//        }
//
//        if let Some(falling_blocks) = &self.falling_blocks {
//            for part in falling_blocks.body.iter() {
//                let params = graphics::DrawParams::new()
//                    .position(Vec2::new(part.x as f32 * part_size, part.y as f32 * part_size))
//                    .scale(scale)
//                    .color(graphics::Color::rgb(0.5, 0.5, 0.5));
//                self.texture.draw(ctx, params); 
//            }
//        }
//
//
//        for fruit in self.fruits.iter() {
//            let params = graphics::DrawParams::new()
//                .position(Vec2::new(fruit.pos.x as f32 * part_size, fruit.pos.y as f32 * part_size))
//                .scale(scale)
//                .color(graphics::Color::rgb(1.0, 0.5, 0.5));
//            self.texture.draw(ctx, params); 
//        }
//
//
//        for y in 0..(MAP_HEIGHT) {
//            for x in 0..(MAP_WIDTH) {
//                if self.map[(x, y)] {
//                    let coords = Vec2::new(x, y);
//                    let params = graphics::DrawParams::new()
//                        .position(Vec2::new(coords.x as f32 * part_size, coords.y as f32 * part_size))
//                        .scale(scale)
//                        .color(graphics::Color::rgb(0.5, 0.5, 0.5));
//                    self.texture.draw(ctx, params); 
//                }
//            }
//        }
//
//        Ok(())
//    }
//
//    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
//        match self.state {
//            PlayState::Normal => {
//                let mut to_remove = Vec::new();
//                for y in 0..(MAP_HEIGHT) {
//                    let mut to_to_remove = true;
//                    for x in 0..(MAP_WIDTH) {
//                        if !self.map[(x, y)] {
//                            to_to_remove = false;
//                        }
//                    }
//                    if to_to_remove {
//                        to_remove.push(y);
//                    }
//                }
//                for y in to_remove.iter() {
//                    for x in 0..(MAP_WIDTH) {
//                        self.map[(x, *y)] = false;
//                    }
//                    for yy in (0..(*y)).rev() {
//                        for x in 0..(MAP_WIDTH) {
//                            self.map[(x, yy + 1)] = self.map[(x, yy)];
//                        }
//                    }
//                }
//
//                if !self.snake.disabled {
//                    let mut fruits_to_retain = Vec::new();
//                    let head = *self.snake.body.first().unwrap();
//                    for fruit in self.fruits.iter_mut() {
//                        if fruit.pos == head {
//                            self.falling_blocks = Some(FallingBlocks::from_snake(&self.snake));
//                            self.snake.disabled = true;
//                            fruits_to_retain.push(false);
//                            self.state = PlayState::Falling;
//                        } else {
//                            fruits_to_retain.push(true);
//                        }
//                    }
//
//                    let mut count = 0;
//                    for bodypart in 1..self.snake.body.len() {
//                        if head == self.snake.body[bodypart] {
//                            count += 1;
//                        }
//                    }
//                    //ugly
//                    let coords = head.into_tuple();
//                    let coords = (coords.0 as usize, coords.1 as usize);
//                    if count == 1 || self.map[coords] {
//                        panic!();
//                    }
//
//                    for key in input::get_keys_pressed(ctx) {
//                        let current_direction = self.snake.direction_queue.last().unwrap();
//                        let next_direction = match key {
//                            Key::Up    => Direction::Up,
//                            Key::Down  => Direction::Down,
//                            Key::Left  => Direction::Left,
//                            Key::Right => Direction::Right,
//                            _ => *current_direction,
//                        };
//
//                        let forbidden_direction = match current_direction {
//                            Direction::Up    => Direction::Down,
//                            Direction::Down  => Direction::Up,
//                            Direction::Left  => Direction::Right,
//                            Direction::Right => Direction::Left,
//                        };
//                        
//                        if next_direction != forbidden_direction &&
//                           next_direction != *current_direction {
//                            self.snake.direction_queue.push(next_direction);
//                        }
//                    }
//
//                    if self.snake.last_move.elapsed() >= Duration::from_millis(SPEED) {
//                        if self.snake.direction_queue.len() > 1 {
//                            self.snake.direction_queue.remove(0);
//                        }
//                        self.snake.direction_queue.truncate(2);
//                        self.snake.update();
//                        self.snake.last_move = Instant::now();
//                    }
//                    let mut iter = fruits_to_retain.iter();
//                    self.fruits.retain(|_| *iter.next().unwrap());
//                }
//            }
//            PlayState::Falling => {
//                let mut fallen = false;
//                if let Some(falling_blocks) = &mut self.falling_blocks {
//                    if falling_blocks.hit_ground {
//                        fallen = true;
//                    }
//
//                    if falling_blocks.last_move.elapsed() >= Duration::from_millis(SPEED / 3) {
//                        falling_blocks.update(&self.map);
//                        falling_blocks.last_move = Instant::now();
//                    }
//                }
//                if fallen {
//                    self.blocks_to_map();
//                    self.falling_blocks = None;
//                    self.snake.disabled = false;
//                    self.snake.length += 1;
//                    self.snake.body.fill(Vec2::new(MAP_WIDTH as i32 / 2, 0));
//                    self.snake.direction_queue[0] = Direction::Down;
//
//
//                    //ugly
//                    let mut coords = (fastrand::usize(0..(MAP_WIDTH as usize)), 
//                                  fastrand::usize(0..(MAP_HEIGHT as usize)));
//                    while self.map[coords] {
//                        coords = (fastrand::usize(0..(MAP_WIDTH as usize)), 
//                                  fastrand::usize(0..(MAP_HEIGHT as usize)));
//                    }
//                    let coords = Vec2::new(coords.0 as i32, coords.1 as i32);
//                    self.fruits.push(Fruit::new(coords));
//                    self.state = PlayState::Normal;
//                }
//            }
//        }
//        Ok(())
//    }
//}
