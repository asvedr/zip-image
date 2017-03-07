use image::*;
use std::io::*;

pub trait DPix : Sized + Pixel<Subpixel = u8> + 'static {
    fn delta(&self, &Self) -> f32;
    fn dp_read<R : Read>(rd : &mut BufReader<R>) -> Result<Self>;
    fn dp_write<W : Write>(&self, wr : &mut BufWriter<W>) -> Result<()>;
}

impl DPix for Luma<u8> {
    fn delta(&self, other : &Luma<u8>) -> f32 {
        ((self.data[0] as f32) - (other.data[0] as f32)) / 255.0
    }
    fn dp_read<A : Read>(rd : &mut BufReader<A>) -> Result<Self> {
        let mut buf : [u8;1] = [0];
        rd.read(&mut buf)?;
        Ok(Luma(buf))
    }
    fn dp_write<A : Write>(&self, wr : &mut BufWriter<A>) -> Result<()> {
        wr.write(&self.data)?;
        Ok(())
    }
}

impl DPix for Rgb<u8> {
    fn delta(&self, other : &Rgb<u8>) -> f32 {
        let mut acc : f32 = 0.0;
        for i in 0 .. 3 {
            acc += (self.data[i] as f32) - (other.data[i] as f32);
        }
        acc / 255.0
    }
    fn dp_read<A : Read>(rd : &mut BufReader<A>) -> Result<Self> {
        let mut buf : [u8;3] = [0,0,0];
        rd.read(&mut buf)?;
        Ok(Rgb(buf))
    }
    fn dp_write<A : Write>(&self, wr : &mut BufWriter<A>) -> Result<()> {
        wr.write(&self.data)?;
        Ok(())
    }
}

impl DPix for Rgba<u8> {
    fn delta(&self, other : &Rgba<u8>) -> f32 {
        let mut acc : f32 = 0.0;
        for i in 0 .. 4 {
            acc += (self.data[i] as f32) - (other.data[i] as f32);
        }
        acc / 255.0
    }
    fn dp_read<A : Read>(rd : &mut BufReader<A>) -> Result<Self> {
        let mut buf : [u8;4] = [0,0,0,0];
        rd.read(&mut buf)?;
        Ok(Rgba(buf))
    }
    fn dp_write<A : Write>(&self, wr : &mut BufWriter<A>) -> Result<()> {
        wr.write(&self.data)?;
        Ok(())
    }
}

impl DPix for LumaA<u8> {
    fn delta(&self, other : &LumaA<u8>) -> f32 {
        let mut acc : f32 = 0.0;
        for i in 0 .. 2 {
            acc += (self.data[i] as f32) - (other.data[i] as f32);
        }
        acc / 255.0
    }
    fn dp_read<A : Read>(rd : &mut BufReader<A>) -> Result<Self> {
        let mut buf : [u8;2] = [0,0];
        rd.read(&mut buf)?;
        Ok(LumaA(buf))
    }
    fn dp_write<A : Write>(&self, wr : &mut BufWriter<A>) -> Result<()> {
        wr.write(&self.data)?;
        Ok(())
    }
}

