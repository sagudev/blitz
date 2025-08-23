use std::fmt::Debug;
use std::io::Cursor;

use anyrender::PaintScene;
use image::ImageReader;
use kurbo::{Affine, Point, Rect, Size};
use peniko::color::palette::css::ORANGE;
use peniko::{Blob, Image};

use crate::svg::Svg;

pub trait Object: Clone + Debug {
    const SIZE: Size;

    fn new() -> Self;

    fn draw(&self, scene: &mut impl PaintScene, pos: Point);
}

#[derive(Clone, Debug)]
struct Primitive(Rect);

impl Object for Primitive {
    const SIZE: Size = Size::new(20., 40.);

    fn new() -> Self {
        Self(Rect::from_origin_size(Point::ORIGIN, Self::SIZE))
    }

    fn draw(&self, scene: &mut impl PaintScene, pos: Point) {
        scene.fill(
            peniko::Fill::NonZero,
            Affine::translate(pos.to_vec2()),
            ORANGE,
            None,
            &self.0,
        );
    }
}

#[derive(Clone, Debug)]
struct Bunny(Image);

static BUNNY_PNG: &[u8] = include_bytes!("./bunny.png");

impl Object for Bunny {
    const SIZE: Size = Size::new(26., 37.);

    fn new() -> Self {
        let raw_bunny_image =
            ImageReader::with_format(Cursor::new(BUNNY_PNG), image::ImageFormat::Png)
                .decode()
                .unwrap()
                .into_rgba8();
        let width = raw_bunny_image.width();
        let height = raw_bunny_image.height();
        assert_eq!(Size::new(width as f64, height as f64), Self::SIZE);
        Self(Image {
            data: Blob::from(raw_bunny_image.into_vec()),
            format: peniko::ImageFormat::Rgba8,
            width,
            height,
            x_extend: peniko::Extend::Pad,
            y_extend: peniko::Extend::Pad,
            quality: peniko::ImageQuality::Low,
            alpha: 1.0,
        })
    }

    fn draw(&self, scene: &mut impl PaintScene, pos: Point) {
        scene.draw_image(&self.0, Affine::translate(pos.to_vec2()));
    }
}

static TIGER_SVG: &[u8] = include_bytes!("./Ghostscript_Tiger.svg");

#[derive(Clone, Debug)]
struct Tiger(Svg);

impl Object for Tiger {
    const SIZE: Size = Size::new(200., 200.);

    fn new() -> Self {
        Self(Svg::parse(TIGER_SVG))
    }

    fn draw(&self, scene: &mut impl PaintScene, pos: Point) {
        self.0.fill_stroke(scene, Affine::translate(pos.to_vec2()));
    }
}

#[derive(Clone, Debug)]
struct TigerUnstroked(Svg);

impl Object for TigerUnstroked {
    const SIZE: Size = Size::new(200., 200.);

    fn new() -> Self {
        Self(Svg::parse(TIGER_SVG))
    }

    fn draw(&self, scene: &mut impl PaintScene, pos: Point) {
        self.0.fill(scene, Affine::translate(pos.to_vec2()));
    }
}

static WORLD_MESH: &[u8] = include_bytes!("./world-mesh.svg");

#[derive(Clone, Debug)]
struct Fill(Svg);

impl Object for Fill {
    const SIZE: Size = Size::new(256., 256.);

    fn new() -> Self {
        Self(Svg::parse(WORLD_MESH))
    }

    fn draw(&self, scene: &mut impl PaintScene, pos: Point) {
        self.0
            .fill(scene, Affine::scale(0.5).then_translate(pos.to_vec2()));
    }
}

#[derive(Clone, Debug)]
struct Stroke(Svg);

impl Object for Stroke {
    const SIZE: Size = Size::new(256., 256.);

    fn new() -> Self {
        Self(Svg::parse(WORLD_MESH))
    }

    fn draw(&self, scene: &mut impl PaintScene, pos: Point) {
        self.0
            .stroke(scene, Affine::scale(0.5).then_translate(pos.to_vec2()));
    }
}
