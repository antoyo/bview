/*
 * Use green arrow to indicate squares or files that are important to cover.
 * Use yellow arrow to indicate plans.
 */

extern crate chessground;
extern crate gdk;
extern crate gtk;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;
extern crate shakmaty;

use std::str::FromStr;

use chessground::{
    DrawBrush,
    DrawShape,
    Ground,
    SetBoard,
    ShapesChanged,
};
use gdk::RGBA;
use gtk::{
    ButtonExt,
    Inhibit,
    LabelExt,
    OrientableExt,
    Orientation::Vertical,
    StateFlags,
    WidgetExt,
};
use relm::Widget;
use relm_derive::widget;
use shakmaty::{Board, Square};

use self::Msg::*;

#[derive(Msg)]
pub enum Msg {
    ShapesDrawn(Vec<DrawShape>),
    Quit,
    Validate,
}

pub struct Model {
    result: String,
    shapes: Vec<DrawShape>,
}

struct Shape {
    orig: Square,
    dest: Square,
    brush: DrawBrush,
}

impl PartialEq<DrawShape> for Shape {
    fn eq(&self, rhs: &DrawShape) -> bool {
        self.orig == rhs.orig() && self.dest == rhs.dest() && self.brush == rhs.brush()
    }
}

#[widget]
impl Widget for Win {
    fn init_view(&mut self) {
        let board = Board::from_str("rnbq1rk1/pp2ppbp/3p1np1/2pP4/4PB2/2N2P2/PPPQ2PP/R3KBNR").expect("board");
        self.ground.emit(SetBoard(board));
    }

    fn model() -> Model {
        Model {
            result: String::new(),
            shapes: vec![],
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            ShapesDrawn(shapes) => self.model.shapes = shapes,
            Quit => gtk::main_quit(),
            Validate => {
                let expected_shapes = vec![
                    Shape {
                        orig: Square::F4,
                        dest: Square::H6,
                        brush: DrawBrush::Yellow,
                    },
                    Shape {
                        orig: Square::E1,
                        dest: Square::C1,
                        brush: DrawBrush::Yellow,
                    },
                    Shape {
                        orig: Square::H6,
                        dest: Square::G7,
                        brush: DrawBrush::Red,
                    },
                ];

                let mut valid = true;
                if expected_shapes.len() != self.model.shapes.len() {
                    valid = false;
                }

                for shape in &self.model.shapes {
                    if !expected_shapes.iter().any(|expected_shape| expected_shape == shape) {
                        valid = false;
                    }
                }

                if valid {
                    self.valid();
                }
                else {
                    self.invalid();
                }
            },
        }
    }

    fn invalid(&mut self) {
        self.model.result = "Invalide".to_string();
        self.label.override_color(StateFlags::NORMAL, Some(&RGBA::red()));
    }

    fn valid(&mut self) {
        self.model.result = "Valid".to_string();
        self.label.override_color(StateFlags::NORMAL, Some(&RGBA::green()));
    }

    view! {
        gtk::Window {
            gtk::Box {
                orientation: Vertical,
                #[name="ground"]
                Ground {
                    ShapesChanged(ref shapes) => ShapesDrawn(shapes.clone()),
                },
                gtk::Button {
                    label: "Valider",
                    clicked => Validate,
                },
                #[name="label"]
                gtk::Label {
                    text: &self.model.result,
                },
            },
            delete_event(_, _) => (Msg::Quit, Inhibit(false)),
        }
    }
}

fn main() {
    Win::run(()).expect("window run");
}
