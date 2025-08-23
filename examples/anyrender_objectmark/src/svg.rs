// Copyright 2023 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::u8;

use anyrender::PaintScene;
use kurbo::{Affine, BezPath, Point, Stroke};
use peniko::{Color, Fill};
use usvg::Tree;

#[derive(Clone, Debug)]
struct SvgPath {
    path: BezPath,
    fill: Option<Color>,
    stroke: Option<Color>,
    stroked: Stroke,
}

#[derive(Clone, Debug)]
pub(crate) struct Svg(Vec<SvgPath>);

impl Svg {
    fn new() -> Self {
        Self(Vec::new())
    }

    pub fn parse(bytes: &[u8]) -> Self {
        let mut s = Self::new();
        let opt = usvg::Options::default();
        let tree = Tree::from_data(bytes, &opt).expect("Failed to parse SVG");
        s.walk(tree.root(), Affine::IDENTITY);
        s
    }

    fn walk(&mut self, svg: &usvg::Group, transform: Affine) {
        let transform = transform * to_affine(&svg.abs_transform());
        for node in svg.children() {
            match node {
                usvg::Node::Group(group) => self.walk(&group, transform),
                usvg::Node::Path(path) => {
                    let mut bez_path = to_bez_path(path);
                    bez_path.apply_affine(transform /* to_affine(path.abs_transform())*/);
                    self.0.push(SvgPath {
                        path: bez_path,
                        fill: path.fill().map(|s| s.paint()).map(to_brush),
                        stroke: path.stroke().map(|s| to_brush(s.paint())),
                        stroked: path
                            .stroke()
                            .map(|s| Stroke::new(s.width().get() as f64))
                            .unwrap_or_default(),
                    });
                }
                usvg::Node::Image(_) => unimplemented!(),
                usvg::Node::Text(_) => unimplemented!(),
            }
        }
    }

    pub fn fill(&self, scene: &mut impl PaintScene, transform: Affine) {
        for svg_path in &self.0 {
            if let Some(brush) = svg_path.fill {
                scene.fill(Fill::NonZero, transform, brush, None, &svg_path.path);
            }
        }
    }

    pub fn stroke(&self, scene: &mut impl PaintScene, transform: Affine) {
        for svg_path in &self.0 {
            if let Some(brush) = svg_path.stroke.or(svg_path.fill) {
                scene.stroke(&svg_path.stroked, transform, brush, None, &svg_path.path);
            }
        }
    }

    pub fn fill_stroke(&self, scene: &mut impl PaintScene, transform: Affine) {
        for svg_path in &self.0 {
            if let Some(brush) = svg_path.fill {
                scene.fill(Fill::NonZero, transform, brush, None, &svg_path.path);
            }
            if let Some(brush) = svg_path.stroke {
                scene.stroke(&svg_path.stroked, transform, brush, None, &svg_path.path);
            }
        }
    }
}

fn to_affine(ts: &usvg::Transform) -> Affine {
    let usvg::Transform {
        sx,
        kx,
        ky,
        sy,
        tx,
        ty,
    } = ts;
    Affine::new([sx, ky, kx, sy, tx, ty].map(|&x| f64::from(x)))
}

fn to_bez_path(path: &usvg::Path) -> BezPath {
    let mut local_path = BezPath::new();
    // The semantics of SVG paths don't line up with `BezPath`; we
    // must manually track initial points
    let mut just_closed = false;
    let mut most_recent_initial = (0., 0.);
    for elt in path.data().segments() {
        match elt {
            usvg::tiny_skia_path::PathSegment::MoveTo(p) => {
                if std::mem::take(&mut just_closed) {
                    local_path.move_to(most_recent_initial);
                }
                most_recent_initial = (p.x.into(), p.y.into());
                local_path.move_to(most_recent_initial);
            }
            usvg::tiny_skia_path::PathSegment::LineTo(p) => {
                if std::mem::take(&mut just_closed) {
                    local_path.move_to(most_recent_initial);
                }
                local_path.line_to(Point::new(p.x as f64, p.y as f64));
            }
            usvg::tiny_skia_path::PathSegment::QuadTo(p1, p2) => {
                if std::mem::take(&mut just_closed) {
                    local_path.move_to(most_recent_initial);
                }
                local_path.quad_to(
                    Point::new(p1.x as f64, p1.y as f64),
                    Point::new(p2.x as f64, p2.y as f64),
                );
            }
            usvg::tiny_skia_path::PathSegment::CubicTo(p1, p2, p3) => {
                if std::mem::take(&mut just_closed) {
                    local_path.move_to(most_recent_initial);
                }
                local_path.curve_to(
                    Point::new(p1.x as f64, p1.y as f64),
                    Point::new(p2.x as f64, p2.y as f64),
                    Point::new(p3.x as f64, p3.y as f64),
                );
            }
            usvg::tiny_skia_path::PathSegment::Close => {
                just_closed = true;
                local_path.close_path();
            }
        }
    }

    local_path
}

fn to_brush(paint: &usvg::Paint) -> Color {
    match paint {
        usvg::Paint::Color(color) => Color::from_rgba8(color.red, color.green, color.blue, u8::MAX),
        _ => unimplemented!(),
    }
}
