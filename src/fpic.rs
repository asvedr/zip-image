use image::*;
use dpix::*;
use rect::*;
use std::io::*;
use std::fs::File;
use std::mem::transmute;

/*
 * развертка:
 * 1 создаем пустую сетку
 * 2 наносим первые пикселы
 * 3 из пикселов по схеме подобия выстраиваем подобные 2x2
 * 4 выстраиваем 4x4 и т.д.
 *
 * сжатие:
 * 1 ищем подобия среди (NxN) подобно (MxM) где M < N
 * 2 для найденных подобий закрашеваем более большие участки как использованные
 * 3 ищем подобия на N-1 ... игнорируем закрашенные учестки
 * 4 когда дошли до 1x1 
 *
 * Если брать строго по квадратной сетке, то при дальнейшем сжатии просто не трогаем пикселы,
 * которые ужа заюзаны на предыдущих этапах.
 *
 */

type Sz = u32;
#[inline(always)]
fn write_sz<A : Write>(a : Sz, w : &mut BufWriter<A>) -> Result<usize> {
    let arr : [u8;4] = unsafe{ transmute(a) };
    w.write(&arr)
}
#[inline(always)]
fn read_sz<A : Read>(r : &mut BufReader<A>) -> Result<Sz> {
    let mut arr : [u8;4] = [0,0,0,0];
    r.read(&mut arr)?;
    unsafe{ Ok(transmute(arr)) }
}

struct Schema { // fractal equal
    small_x  : Sz,
    small_y  : Sz,
    small_wh : Sz,
    //big_x    : Sz,
    //big_y    : Sz,
    big_wh   : Sz,
    bigs     : Vec<(Sz,Sz)>
}

// Clr is [u8;1] or [u8;3] or [u8;4]
struct Dot<Clr> {
    x   : Sz,
    y   : Sz,
    rgb : Clr
}

pub struct ZImage<Px> {
    schemas : Vec<Schema>,
    width   : Sz,
    height  : Sz,
    pixels  : Vec<Dot<Px>>
}

#[inline]
fn is_like<P : DPix>(pic : &ImageBuffer<P,Vec<u8>>, rect_s : &Rect, rect_b : &Rect, n : u32, allow_error : f32) -> bool {
    let sx = rect_s.x;
    let sy = rect_s.y;
    for dx in 0 .. rect_s.w {
        let s_x = sx + dx;
        let b_x = rect_b.x + (dx * n);
        for dy in 0 .. rect_s.h {
            let s_y = sy + dy;
            let b_y = rect_b.y + (dy * n);
            let s_pix = pic.get_pixel(s_x, s_y);
            for nx in 0 .. n {
                for ny in 0 .. n {
                    if s_pix.delta(pic.get_pixel(b_x + nx, b_y + ny)) > allow_error {
                        return false;
                    }
                }
            }
        }
    }
    true
}

fn zip_rec<Px : DPix>(img : &ImageBuffer<Px,Vec<u8>>, zipped : &mut Vec<Rect>, nzipped : &mut Vec<Rect>, schemas : &mut Vec<Schema>) {
    let width = img.width();
    let height = img.height();
    //let mut pix_to_write : Vec<Dot<Px>> = Vec::new();
    let mut rect_s = Rect {
        x : 0,
        y : 0,
        w : 0,
        h : 0
    };
    let small_sq : u32 = 3;
    let big_sq   : u32 = 6;
    for x in 0 .. width / big_sq {
        let big_x = x * big_sq;
        'big_loop : for y in 0 .. height / big_sq {
            let big_y = y * big_sq;
            let mut found = false;
            let rect_b = Rect {
                x : big_x,
                y : big_y,
                w : big_sq,
                h : big_sq
            };
            for sch in schemas.iter() {
                if sch.small_x >= big_x && sch.small_x + sch.small_wh < (x + 1) * big_sq && sch.small_y >= big_y && sch.small_y + sch.small_wh < (y + 1) * big_sq {
                    nzipped.push(rect_b);
                    continue 'big_loop;
                }
            }
            'loc_loop : for x in (0 .. small_sq).rev() {
                let small_x = x * small_sq;
                for y in (0 .. small_sq).rev() {
                    let small_y = y * small_sq;
                    if small_x >= big_x && small_x < big_x + big_sq && small_y >= big_y && small_y < big_y + big_sq {
                        continue
                    }
                    rect_s.x = small_x;
                    rect_s.y = small_y;
                    rect_s.w = small_sq;
                    rect_s.h = small_sq;
                    if is_like(img, &rect_s, &rect_b, 2, 0.5) {
                        let mut index = None;
                        for i in 0 .. schemas.len() {
                            if schemas[i].small_x == small_x && schemas[i].small_y == small_y {
                                index = Some(i);
                                break
                            }
                        }
                        match index {
                            None => {
                                schemas.push(Schema{
                                    small_x  : small_x,
                                    small_y  : small_y,
                                    small_wh : small_sq,
                                    //big_x    : big_x,
                                    //big_y    : big_y,
                                    big_wh   : big_sq,
                                    bigs     : vec![(big_x, big_y)]
                                })
                            },
                            Some(i) => {
                                schemas[i].bigs.push((big_x, big_y))
                            }
                        }
                        found = true;
                        break 'loc_loop;
                        //zipped.push(rect_b);
                    } else {
                        //nzipped.push(rect_b);
                    }
                }
            }
            if found {
                zipped.push(rect_b)
            }
            else {
                nzipped.push(rect_b)
            }
        }
    }
}

impl<Px : DPix + Eq> ZImage<Px> {
    pub fn where_neq(&self, other : &ZImage<Px>) -> Option<String> {
        macro_rules! ans{($a:expr) => {return Some(format!("{}",$a))};}
        if self.width != other.width {
            ans!("width")
        }
        if self.height != other.height {
            ans!("height")
        }
        if self.schemas.len() != other.schemas.len() {
            ans!("sch len")
        }
        if self.pixels.len() != other.pixels.len() {
            ans!("pix len")
        }
        for i in 0 .. self.schemas.len() {
            let a = &self.schemas[i];
            let b = &other.schemas[i];
            if a.small_x != b.small_x {
                ans!(format!("sch x small {}", i))
            }
            if a.small_y != b.small_y {
                ans!(format!("sch x small {}", i))
            }
            if a.small_wh != b.small_wh {
                ans!(format!("sch x small {}", i))
            }
            if a.big_wh != b.big_wh {
                ans!(format!("sch x small {}", i))
            }
            if a.bigs.len() != b.bigs.len() {
                ans!(format!("sch x small {}", i))
            }
            for j in 0 .. a.bigs.len() {
                //let a = &a.bigs[i];
                //let b = &b.bigs[i];
                if a.bigs[i] != b.bigs[i] {
                    ans!(format!("sch bigs {} of {}", j, i))
                }
            }
            for i in 0 .. self.pixels.len() {
                let a = &self.pixels[i];
                let b = &other.pixels[i];
                if a.x != b.x {
                    ans!(format!("pix x {}", i))
                }
                if a.y != b.y {
                    ans!(format!("pix y {}", i))
                }
                if a.rgb != b.rgb {
                    ans!(format!("pix val {} (len: {})", i, self.pixels.len()))
                }
            }
        }
        None
    }
    pub fn zip(img : &ImageBuffer<Px,Vec<u8>>) -> ZImage<Px> {
        let mut sch = vec![];
        let mut zpd = vec![];
        let mut nzpd = vec![];
        zip_rec(img, &mut zpd, &mut nzpd, &mut sch);
        println!("zippped: {}\nnot zipped: {}\nblocks: {}", zpd.len(), nzpd.len(), sch.len());
        let mut pixels = vec![];
        for rect in nzpd {
            for x in rect.x .. rect.x + rect.w {
                for y in rect.y .. rect.y + rect.h {
                    pixels.push(Dot{x : x, y : y, rgb : img.get_pixel(x,y).clone()})
                }
            }
        }
        ZImage {
            schemas : sch,
            width   : img.width(),
            height  : img.height(),
            pixels  : pixels
        }
    }
    pub fn unzip(&self) -> ImageBuffer<Px,Vec<u8>> {
        let mut img : ImageBuffer<Px, Vec<u8>> = ImageBuffer::new(self.width, self.height);
        for dot in self.pixels.iter() {
            let px = img.get_pixel_mut(dot.x, dot.y);
            *px = dot.rgb;
        }
        //let len = self.schemas.len();
        let mut pin = 0;
        let mut pout = 0;
        for sch in self.schemas.iter().rev() {
            let swh = sch.small_wh;// as usize;
            let bwh = sch.big_wh;// as usize;
            let sx  = sch.small_x;// as usize;
            let sy  = sch.small_y;// as usize;
            let d = bwh / swh;
            for x in 0 .. swh {
                let sx = sx + x;
                for y in 0 .. swh {
                    let sy = sy + y;
                    let val = img.get_pixel(sx, sy).clone();
                    let mut found = false;
                    for d in self.pixels.iter() {
                        if d.x == sx && d.y == sy {
                            found = true;
                            break;
                        }
                    }
                    if found {
                        pin += 1;
                    } else {
                        pout += 1;
                    }
                    for dx in 0 .. d {
                        for dy in 0 .. d {
                            for big in sch.bigs.iter() {
                                match *big {
                                    (ref bx, ref by) => {
                                        let px = img.get_pixel_mut(bx + (x * d) + dx, by + (y * d) + dy);
                                        *px = val.clone()
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        println!("{} / {}", pin, pout);
        img
    }
    pub fn save(&self, name : &str) -> Result<()> {
        let mut wrt = BufWriter::new(File::create(name)?);
        write_sz(self.width, &mut wrt)?;
        write_sz(self.height, &mut wrt)?;
        write_sz(self.schemas.len() as Sz, &mut wrt)?;
        write_sz(self.pixels.len() as Sz, &mut wrt)?;
        for s in self.schemas.iter() {
            write_sz(s.small_x, &mut wrt)?;
            write_sz(s.small_y, &mut wrt)?;
            write_sz(s.small_wh, &mut wrt)?;
            write_sz(s.big_wh, &mut wrt)?;
            write_sz(s.bigs.len() as Sz, &mut wrt)?;
            for b in s.bigs.iter() {
                match *b {
                    (ref x, ref y) => {
                        write_sz(*x, &mut wrt)?;
                        write_sz(*y, &mut wrt)?;
                    }
                }
            }
        }
        for p in self.pixels.iter() {
            write_sz(p.x, &mut wrt)?;
            write_sz(p.y, &mut wrt)?;
            p.rgb.dp_write(&mut wrt)?;
        }
        Ok(())
    }
    pub fn load(name : &str) -> Result<ZImage<Px>> {
        let mut rdr = BufReader::new(File::open(name)?);
        let mut slf = ZImage {
            width : 0,
            height : 0,
            schemas : vec![],
            pixels : vec![]
        };
        slf.width = read_sz(&mut rdr)?;
        slf.height = read_sz(&mut rdr)?;
        let schcnt = read_sz(&mut rdr)?;
        let pxcnt = read_sz(&mut rdr)?;
        slf.schemas.reserve(schcnt as usize);
        for _ in 0 .. schcnt {
            let small_x  = read_sz(&mut rdr)?;
            let small_y  = read_sz(&mut rdr)?;
            let small_wh = read_sz(&mut rdr)?;
            let big_wh   = read_sz(&mut rdr)?;
            let blen     = read_sz(&mut rdr)?;
            let mut bigs = Vec::new();
            for _ in 0 .. blen {
                let x = read_sz(&mut rdr)?;
                let y = read_sz(&mut rdr)?;
                bigs.push((x,y));
            }
            slf.schemas.push(Schema{
                small_x  : small_x,
                small_y  : small_y,
                small_wh : small_wh,
                big_wh   : big_wh,
                bigs     : bigs
            });
        }
        for _ in 0 .. pxcnt {
            let x = read_sz(&mut rdr)?;
            let y = read_sz(&mut rdr)?;
            let px = Dot {
                x   : x,
                y   : y,
                rgb : DPix::dp_read(&mut rdr)?//read_sz(&mut rdr)?
            };
            slf.pixels.push(px);
        }
        Ok(slf)
    }
}
