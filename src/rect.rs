pub struct Rect {
    pub x : u32,
    pub y : u32,
    pub w : u32,
    pub h : u32
}

impl Rect {
    #[inline(always)]
    fn is_in(&self, x : u32, y : u32) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }
}
