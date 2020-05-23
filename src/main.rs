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

use std::cmp::min;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use chessground::{
    DrawBrush,
    DrawShape,
    Ground,
    GroundMsg,
    SetBoard,
};
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
    ToolButtonExt,
    WidgetExt,
};
use pgn_reader::{
    BufferedReader,
    SanPlus,
    Visitor,
};
use relm::Widget;
use relm_derive::widget;
use shakmaty::{
    Board,
    Chess,
    Position,
    Setup,
    Square,
};

use self::Msg::*;

#[derive(Msg)]
pub enum Msg {
    Flip,
    ImportPGN,
    NextMove,
    PreviousMove,
    Quit,
}

#[derive(Clone)]
struct TrainingPosition {
    annotations: Vec<Shape>,
    position: Board,
}

pub struct Model {
    current_position: usize,
    result: String,
    game: Vec<Chess>,
}

#[derive(Clone)]
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
    fn model() -> Model {
        Model {
            current_position: 0,
            result: String::new(),
            game: vec![],
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Flip => self.ground.emit(GroundMsg::Flip),
            ImportPGN => {
                let dialog = FileChooserDialog::with_buttons(
                    Some("Select a PGN file to import"),
                    Some(&self.window),
                    FileChooserAction::Open,
                    &[("Import", ResponseType::Ok), ("Cancel", ResponseType::Cancel)],
                );
                if dialog.run() == ResponseType::Ok {
                    for filename in dialog.get_filenames() {
                        if let Err(error) = self.import_file(&filename) {
                            let message_dialog = MessageDialog::new(Some(&self.window), DialogFlags::empty(), MessageType::Error, ButtonsType::Ok, &error);
                            message_dialog.run();
                            message_dialog.destroy();
                        }
                    }
                }
                dialog.destroy();
            },
            NextMove => {
                self.model.current_position = min(self.model.current_position + 1, self.model.game.len() - 1);
                self.show_position();
            },
            PreviousMove => {
                if self.model.current_position > 0 {
                    self.model.current_position -= 1;
                }
                self.show_position();
            },
            Quit => gtk::main_quit(),
        }
    }

    fn import_file(&mut self, filename: &PathBuf) -> Result<(), String> {
        let mut file = File::open(filename).map_err(|error| error.to_string())?;
        let mut data = vec![];
        file.read_to_end(&mut data).map_err(|error| error.to_string())?;
        let (result, _, _) = encoding_rs::WINDOWS_1252.decode(&data);

        let mut importer = FENImporter::new();
        let mut reader = BufferedReader::new_cursor(result.as_bytes());
        reader.read_all(&mut importer).map_err(|_| "Cannot parse PGN file")?;
        self.model.game = importer.game();
        self.model.current_position = 0;
        self.show_position();
        Ok(())
    }

    fn show_position(&self) {
        if let Some(position) = self.model.game.get(self.model.current_position) {
            self.ground.emit(SetBoard(position.board().clone()));
        }
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
                        icon_name: Some("object-flip-vertical"),
                        label: Some("Flip board"),
                        clicked => Flip,
                    },
                    gtk::ToolButton {
                        icon_name: Some("application-exit"),
                        label: Some("Quit"),
                        clicked => Quit,
                    },
                },
                #[name="ground"]
                Ground {
                },
                gtk::ButtonBox {
                    gtk::Button {
                        label: "Précédent",
                        clicked => PreviousMove,
                    },
                    gtk::Button {
                        label: "Suivant",
                        clicked => NextMove,
                    },
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

struct FENImporter {
    position: Chess,
    game: Vec<Chess>,
}

impl FENImporter {
    fn new() -> Self {
        Self {
            position: Chess::default(),
            game: vec![],
        }
    }

    fn game(&mut self) -> Vec<Chess> {
        self.game.clone()
    }
}

impl Visitor for FENImporter {
    type Result = ();

    fn begin_game(&mut self) {
        self.position = Chess::default();
        self.game.push(self.position.clone());
    }

    fn end_game(&mut self) -> Self::Result {
    }

    fn san(&mut self, san_plus: SanPlus) {
        if let Ok(mov) = san_plus.san.to_move(&self.position) {
            self.position.play_unchecked(&mov);
            self.game.push(self.position.clone());
        }
        else {
            eprintln!("{:?}", san_plus.san.to_move(&self.position));
        }
    }
}

fn main() {
    Win::run(()).expect("window run");
}
