use macroquad::prelude::*;
pub enum ShapeKind {
    Circle {
        radius: f32
    },
    Rect {
        width: f32, height: f32
    }
}

pub enum Collider {
    Circle(Circle),
    Rect(Rect),
}

pub struct Shape {
    pub size: f32,
    pub speed: f32,
    pub x: f32,
    pub y: f32,
    pub color: Color,
    pub kind: ShapeKind,
    pub collided: bool,
    pub active: bool
}

impl Default for Shape {
    fn default() -> Self {
        Self {
            size: 0.0,
            speed: 0.0,
            x: 0.0,
            y: 0.0,
            color: WHITE,
            kind: ShapeKind::Circle { radius: 16.0 },
            collided: false,
            active: false, // Default-nya tidak aktif!
        }
    }
}

impl Collider  {
    pub fn overlaps(&self, other: &Collider) -> bool {
        match (self, other){
            (Collider::Circle(c1), Collider::Circle(c2)) => c1.overlaps(c2),
            (Collider::Rect(r1), Collider::Rect(r2)) => r1.overlaps(r2),
            (Collider::Circle(c), Collider::Rect(r)) => c.overlaps_rect(r),
            (Collider::Rect(r), Collider::Circle(c)) => c.overlaps_rect(r),
        }
    }
}

impl Shape {
    pub fn collides_with(&mut self, other: &Self) -> bool {
        self.collider().overlaps(&other.collider())
    }

    pub fn draw(&self){
        match self.kind{
            ShapeKind::Circle { radius } => {
                draw_circle(self.x, self.y, radius, self.color);
            }
            ShapeKind::Rect { width, height } => {
                draw_rectangle(
                    self.x - width / 2.0,
                    self.y - height / 2.0,
                    width,
                    height,
                    self.color,
                );
            }
        }
    }

    pub fn collider(&self) -> Collider {
        match self.kind {
            ShapeKind::Circle { radius } => {
                Collider::Circle(Circle::new(self.x, self.y, radius))
            },
            ShapeKind::Rect { width, height } => {
                Collider::Rect(Rect::new(
                    self.x - width / 2.0,
                    self.y - height / 2.0,
                    width,
                    height
                ))
            }
        }
    }

    pub fn new_inactive() -> Self{
        Self { active:false,..Default::default() }
    }

    pub fn new() ->Self{
        Self {..Default::default() }
    }
}
