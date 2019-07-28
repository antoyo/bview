/*
 * Use green arrow to indicate squares or files that are important to cover.
 * Use yellow arrow to indicate plans.
 *
 * TODO: prevent the default mouse behaviour of the chessboard and emit these events with the left
 * click.
 */

extern crate chessground;
extern crate encoding_rs;
extern crate gdk;
extern crate gtk;
extern crate pgn_reader;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;
extern crate shakmaty;
extern crate sqlite;

mod parser;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
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
    ButtonsType,
    DialogExt,
    DialogFlags,
    FileChooserAction,
    FileChooserDialog,
    FileChooserExt,
    Inhibit,
    LabelExt,
    MessageDialog,
    MessageType,
    OrientableExt,
    Orientation::Vertical,
    ResponseType,
    StateFlags,
    ToolButtonExt,
    WidgetExt,
};
use pgn_reader::{
    BufferedReader,
    RawComment,
    SanPlus,
    Skip,
    Visitor,
};
use relm::Widget;
use relm_derive::widget;
use shakmaty::{
    Board,
    Chess,
    Position,
    Square,
};
use shakmaty::fen::fen;

use parser::parse_annotations;
use self::Msg::*;

#[derive(Msg)]
pub enum Msg {
    ImportPGN,
    Quit,
    ShapesDrawn(Vec<DrawShape>),
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
            ImportPGN => {
                let dialog = FileChooserDialog::with_buttons(
                    Some("Select a PGN file to import"),
                    Some(&self.window),
                    FileChooserAction::Open,
                    &[("Import", ResponseType::Ok), ("Cancel", ResponseType::Cancel)],
                );
                if dialog.run() == ResponseType::Ok {
                    for filename in dialog.get_filenames() {
                        if let Err(error) = import_file(&filename) {
                            let message_dialog = MessageDialog::new(Some(&self.window), DialogFlags::empty(), MessageType::Error, ButtonsType::Ok, &error);
                            message_dialog.run();
                            message_dialog.destroy();
                        }
                    }
                }
                dialog.destroy();
            },
            Quit => gtk::main_quit(),
            ShapesDrawn(shapes) => self.model.shapes = shapes,
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
        #[name="window"]
        gtk::Window {
            gtk::Box {
                orientation: Vertical,
                gtk::Toolbar {
                    gtk::ToolButton {
                        icon_name: Some("document-open"),
                        label: Some("Import PGN files"),
                        clicked => ImportPGN,
                    },
                    gtk::ToolButton {
                        icon_name: Some("media-playback-start"),
                        label: Some("Start training"),
                    },
                    gtk::ToolButton {
                        icon_name: Some("application-exit"),
                        label: Some("Quit"),
                        clicked => Quit,
                    },
                },
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
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

fn import_file(filename: &PathBuf) -> Result<(), String> {
    let mut file = File::open(filename).map_err(|error| error.to_string())?;
    let mut data = vec![];
    file.read_to_end(&mut data).map_err(|error| error.to_string())?;
    let (result, _, _) = encoding_rs::WINDOWS_1252.decode(&data);

    let mut importer = FENImporter::new();
    let mut reader = BufferedReader::new_cursor(result.as_bytes());
    reader.read_all(&mut importer).map_err(|_| "Cannot parse PGN file")?;
    Ok(())
}

struct FENImporter {
    current_stack: Vec<Chess>,
    position: Chess,
    previous_position: Chess,
    previous_stack: Vec<Chess>,
}

impl FENImporter {
    fn new() -> Self {
        Self {
            current_stack: vec![],
            position: Chess::default(),
            previous_position: Chess::default(),
            previous_stack: vec![],
        }
    }
}

impl Visitor for FENImporter {
    type Result = ();

    fn begin_game(&mut self) {
        println!("Begin game");
        self.position = Chess::default();
    }

    fn begin_variation(&mut self) -> Skip {
        self.current_stack.push(self.position.clone());
        self.previous_stack.push(self.previous_position.clone());
        self.position = self.previous_stack.last().cloned().expect("previous stack top");
        Skip(false)
    }

    fn end_game(&mut self) -> Self::Result {
        println!("End game");
        ()
    }

    fn end_variation(&mut self) {
        self.position = self.current_stack.pop().expect("current stack");
        self.previous_position = self.previous_stack.pop().expect("previous stack");
    }

    fn comment(&mut self, comment: RawComment) {
        let annotations = parse_annotations(comment.as_bytes());
        if !annotations.is_empty() {
            println!("FEN: {}", fen(&self.position));
            println!("{:?}", annotations);
        }
    }

    fn san(&mut self, san_plus: SanPlus) {
        if let Ok(mov) = san_plus.san.to_move(&self.position) {
            self.previous_position = self.position.clone();
            self.position.play_unchecked(&mov);
        }
        else {
            eprintln!("Cannot convert san to move {:?}", san_plus);
        }
    }
}

fn main() {
    Win::run(()).expect("window run");
}
