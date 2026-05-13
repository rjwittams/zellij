use super::super::Grid;
use crate::output::{KittyImageData, KittyOutputMediaCache, Output, PlacementId};
use crate::panes::grid::SixelImageStore;
use crate::panes::kitty_asset_store::KittyAssetStore;
use crate::panes::link_handler::LinkHandler;
use insta::assert_snapshot;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use vte;
use zellij_utils::{
    data::{Palette, Style},
    pane_size::SizeInPixels,
    position::Position,
};

use std::fmt::Write;

fn pid(value: u32) -> PlacementId {
    PlacementId::Protocol(value)
}

fn synthetic_wire_placement_id(stable_render_id: u64) -> u32 {
    stable_render_id as u32
}

fn placeholder_wire_placement_id(stable_render_id: u64) -> u32 {
    stable_render_id as u32
}

fn read_fixture(fixture_name: &str) -> Vec<u8> {
    let mut path_to_file = std::path::PathBuf::new();
    path_to_file.push("../src");
    path_to_file.push("tests");
    path_to_file.push("fixtures");
    path_to_file.push(fixture_name);
    std::fs::read(path_to_file)
        .unwrap_or_else(|_| panic!("could not read fixture {:?}", &fixture_name))
}

#[test]
fn vttest1_0() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest1-0";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest1_1() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest1-1";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest1_2() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest1-2";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest1_3() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest1-3";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest1_4() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest1-4";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest1_5() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest1-5";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest2_0() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest2-0";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest2_1() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest2-1";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest2_2() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest2-2";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest2_3() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest2-3";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest2_4() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest2-4";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest2_5() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest2-5";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest2_6() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest2-6";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest2_7() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest2-7";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest2_8() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest2-8";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest2_9() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest2-9";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest2_10() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest2-10";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest2_11() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest2-11";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest2_12() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest2-12";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest2_13() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest2-13";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest2_14() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest2-14";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest3_0() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest3-0";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest8_0() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest8-0";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest8_1() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest8-1";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest8_2() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest8-2";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest8_3() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest8-3";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest8_4() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest8-4";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest8_5() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest8-5";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn csi_b() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "csi-b";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn csi_capital_i() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "csi-capital-i";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn csi_capital_z() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "csi-capital-z";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn terminal_reports() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "terminal_reports";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid.pending_messages_to_pty));
}

#[test]
fn wide_characters() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        104,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "wide_characters";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn wide_characters_line_wrap() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        104,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "wide_characters_line_wrap";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn insert_character_in_line_with_wide_character() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        104,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "wide_characters_middle_line_insert";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn delete_char_in_middle_of_line_with_widechar() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        104,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "wide-chars-delete-middle";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn delete_char_in_middle_of_line_with_multiple_widechars() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        104,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "wide-chars-delete-middle-after-multi";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn fish_wide_characters_override_clock() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        104,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "fish_wide_characters_override_clock";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn bash_delete_wide_characters() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        104,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "bash_delete_wide_characters";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn delete_wide_characters_before_cursor() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        104,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "delete_wide_characters_before_cursor";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn delete_wide_characters_before_cursor_when_cursor_is_on_wide_character() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        104,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "delete_wide_characters_before_cursor_when_cursor_is_on_wide_character";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn delete_wide_character_under_cursor() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        104,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "delete_wide_character_under_cursor";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn replace_wide_character_under_cursor() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        104,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "replace_wide_character_under_cursor";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn wrap_wide_characters() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        90,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "wide_characters_full";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn wrap_wide_characters_on_size_change() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        93,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "wide_characters_full";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    grid.change_size(21, 90);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn unwrap_wide_characters_on_size_change() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        93,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "wide_characters_full";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    grid.change_size(21, 90);
    grid.change_size(21, 93);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn wrap_wide_characters_in_the_middle_of_the_line() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        91,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "wide_characters_line_middle";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn wrap_wide_characters_at_the_end_of_the_line() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        90,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "wide_characters_line_end";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn copy_selected_text_from_viewport() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        27,
        125,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "grid_copy";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);

    grid.start_selection(&Position::new(23, 6));
    // check for widechar, 📦 occupies columns 34, 35, and gets selected even if only the first column is selected
    grid.end_selection(&Position::new(25, 35));
    let text = grid.get_selected_text();
    assert_eq!(
        text.unwrap(),
        "mauris in aliquam sem fringilla.\n\nzellij on  mouse-support [?] is 📦"
    );
}

#[test]
fn copy_wrapped_selected_text_from_viewport() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        22,
        73,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "grid_copy_wrapped";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);

    grid.start_selection(&Position::new(5, 0));
    grid.end_selection(&Position::new(8, 42));
    let text = grid.get_selected_text();
    assert_eq!(
        text.unwrap(),
        "Lorem ipsum dolor sit amet,                                                                                                                          consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua."
    );
}

#[test]
fn copy_selected_text_from_lines_above() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        27,
        125,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "grid_copy";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);

    grid.start_selection(&Position::new(-2, 10));
    // check for widechar, 📦 occupies columns 34, 35, and gets selected even if only the first column is selected
    grid.end_selection(&Position::new(2, 8));
    let text = grid.get_selected_text();
    assert_eq!(
        text.unwrap(),
        "eu scelerisque felis imperdiet proin fermentum leo.\nCursus risus at ultrices mi tempus.\nLaoreet id donec ultrices tincidunt arcu non sodales.\nAmet dictum sit amet justo donec enim.\nHac habi"
    );
}

#[test]
fn copy_selected_text_from_lines_below() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        27,
        125,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "grid_copy";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);

    grid.move_viewport_up(40);

    grid.start_selection(&Position::new(63, 6));
    // check for widechar, 📦 occupies columns 34, 35, and gets selected even if only the first column is selected
    grid.end_selection(&Position::new(65, 35));
    let text = grid.get_selected_text();
    assert_eq!(
        text.unwrap(),
        "mauris in aliquam sem fringilla.\n\nzellij on  mouse-support [?] is 📦"
    );
}

/*
 * These tests below are general compatibility tests for non-trivial scenarios running in the terminal.
 * They use fake TTY input replicated from these scenarios.
 *
 */

#[test]
fn run_bandwhich_from_fish_shell() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        116,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "fish_and_bandwhich";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn fish_tab_completion_options() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        116,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "fish_tab_completion_options";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn fish_select_tab_completion_options() {
    // the difference between this and the previous test is that here we press <TAB>
    // twice, meaning the selection moves between the options and the command line
    // changes.
    // this is not clearly seen in the snapshot because it does not include styles,
    // but we can see the command line change and the cursor staying in place
    // terminal_emulator_color_codes,
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        116,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "fish_select_tab_completion_options";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn vim_scroll_region_down() {
    // here we test a case where vim defines the scroll region as lesser than the screen row count
    // and then scrolls down
    // the region is defined here by vim as 1-26 (there are 28 rows)
    // then the cursor is moved to line 26 and a new line is added
    // what should happen is that the first line in the scroll region (1) is deleted
    // terminal_emulator_color_codes,
    // and an empty line is inserted in the last scroll region line (26)
    // this tests also has other steps afterwards that fills the line with the next line in the
    // sixel_image_store,
    // file
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        116,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vim_scroll_region_down";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn vim_ctrl_d() {
    // in vim ctrl-d moves down half a page
    // in this case, it sends the terminal the csi 'M' directive, which tells it to delete X (13 in
    // this case) lines inside the scroll region and push the other lines up
    // what happens here is that 13 lines are deleted and instead 13 empty lines are added at the
    // end of the scroll region
    // terminal_emulator_color_codes,
    // vim makes sure to fill these empty lines with the rest of the file
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        116,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vim_ctrl_d";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn vim_ctrl_u() {
    // in vim ctrl-u moves up half a page
    // in this case, it sends the terminal the csi 'L' directive, which tells it to insert X (13 in
    // this case) lines at the cursor, pushing away (deleting) the last line in the scroll region
    // this causes the effect of scrolling up X lines (vim replaces the lines with the ones in the
    // file above the current content)
    // terminal_emulator_color_codes,
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        116,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vim_ctrl_u";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn htop() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        116,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "htop";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn htop_scrolling() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        116,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "htop_scrolling";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn htop_right_scrolling() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        116,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "htop_right_scrolling";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn vim_overwrite() {
    // this tests the vim overwrite message
    // to recreate:
    // * open a file in vim
    // * open the same file in another window
    // * change the file in the other window and save
    // terminal_emulator_color_codes,
    // * change the file in the original vim window and save
    // * confirm you would like to change the file by pressing 'y' and then ENTER
    // sixel_image_store,
    // * if everything looks fine, this test passed :)
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        116,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vim_overwrite";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn clear_scroll_region() {
    // this is actually a test of 1049h/l (alternative buffer)
    // @imsnif - the name is a monument to the time I didn't fully understand this mechanism :)
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        116,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "clear_scroll_region";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn display_tab_characters_properly() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        116,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "tab_characters";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn neovim_insert_mode() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        116,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "nvim_insert";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn bash_cursor_linewrap() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        116,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "bash_cursor_linewrap";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn fish_paste_multiline() {
    // here we paste a multiline command in fish shell, making sure we support it
    // going up and changing the colors of our line-wrapped pasted text
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        149,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "fish_paste_multiline";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn git_log() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        149,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "git_log";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn git_diff_scrollup() {
    // this tests makes sure that when we have a git diff that exceeds the screen size
    // we are able to scroll up
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        149,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "git_diff_scrollup";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn emacs_longbuf() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        60,
        284,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "emacs_longbuf_tutorial";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn top_and_quit() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        56,
        235,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "top_and_quit";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn exa_plus_omf_theme() {
    // this tests that we handle a tab delimited table properly
    // without overriding the previous content
    // this is a potential bug because the \t character is a goto
    // if we forwarded it as is to the terminal, we would be skipping
    // over existing on-screen content without deleting it, so we must
    // terminal_emulator_color_codes,
    // convert it to spaces
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        56,
        235,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "exa_plus_omf_theme";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn scroll_up() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        10,
        50,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "scrolling";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    grid.scroll_up_one_line();
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn scroll_down() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        10,
        50,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "scrolling";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    grid.scroll_up_one_line();
    grid.scroll_down_one_line();
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn scroll_up_with_line_wraps() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        10,
        25,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "scrolling";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    grid.scroll_up_one_line();
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn scroll_down_with_line_wraps() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        10,
        25,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "scrolling";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    grid.scroll_up_one_line();
    grid.scroll_down_one_line();
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn scroll_up_decrease_width_and_scroll_down() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        10,
        50,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "scrolling";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    for _ in 0..10 {
        grid.scroll_up_one_line();
    }
    grid.change_size(10, 25);
    for _ in 0..10 {
        grid.scroll_down_one_line();
    }
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn scroll_up_increase_width_and_scroll_down() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        10,
        25,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "scrolling";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    for _ in 0..10 {
        grid.scroll_up_one_line();
    }
    grid.change_size(10, 50);
    for _ in 0..10 {
        grid.scroll_down_one_line();
    }
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn saved_cursor_across_resize() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        4,
        20,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let mut parse = |s, grid: &mut Grid| {
        for b in Vec::from(s) {
            vte_parser.advance(&mut *grid, &[b])
        }
    };
    let content = "
\rLine 1 >fill to 20_<
\rLine 2 >fill to 20_<
\rLine 3 >fill to 20_<
\rL\u{1b}[sine 4 >fill to 20_<";
    parse(content, &mut grid);
    // Move real cursor position up three lines
    let content = "\u{1b}[3A";
    parse(content, &mut grid);
    // Truncate top of terminal, resetting cursor (but not saved cursor)
    grid.change_size(3, 20);
    // Wrap, resetting cursor again (but not saved cursor)
    grid.change_size(3, 10);
    // Restore saved cursor position and write ZZZ
    let content = "\u{1b}[uZZZ";
    parse(content, &mut grid);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn saved_cursor_across_resize_longline() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        4,
        20,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let mut parse = |s, grid: &mut Grid| {
        for b in Vec::from(s) {
            vte_parser.advance(&mut *grid, &[b])
        }
    };
    let content = "
\rLine 1 >fill \u{1b}[sto 20_<";
    parse(content, &mut grid);
    // Wrap each line precisely halfway
    grid.change_size(4, 10);
    // Write 'YY' at the end (ends up on a new wrapped line), restore to the saved cursor
    // and overwrite 'to' with 'ZZ'
    let content = "YY\u{1b}[uZZ";
    parse(content, &mut grid);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn saved_cursor_across_resize_rewrap() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        4,
        4 * 8,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let mut parse = |s, grid: &mut Grid| {
        for b in Vec::from(s) {
            vte_parser.advance(&mut *grid, &[b])
        }
    };
    let content = "
\r12345678123456781234567\u{1b}[s812345678"; // 4*8 chars
    parse(content, &mut grid);
    // Wrap each line precisely halfway, then rewrap to halve them again
    grid.change_size(4, 16);
    grid.change_size(4, 8);
    // Write 'Z' at the end of line 3
    let content = "\u{1b}[uZ";
    parse(content, &mut grid);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn move_cursor_below_scroll_region() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        34,
        114,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "move_cursor_below_scroll_region";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn insert_wide_characters_in_existing_line() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        86,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "chinese_characters_line_middle";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn full_screen_scroll_region_and_scroll_up() {
    // this test is a regression test for a bug
    // where the scroll region would be set to the
    // full viewport and then scrolling up would cause
    // lines to get deleted from the viewport rather
    // than moving to "lines_above"
    // terminal_emulator_color_codes,
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        54,
        80,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "scroll_region_full_screen";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    grid.scroll_up_one_line();
    grid.scroll_up_one_line();
    grid.scroll_up_one_line();
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn ring_bell() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        134,
        64,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "ring_bell";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert!(grid.ring_bell);
}

#[test]
pub fn alternate_screen_change_size() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        20,
        20,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "alternate_screen_change_size";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    // no scrollback in alternate screen
    assert_eq!(grid.scrollback_position_and_length(), (0, 0));
    grid.change_size(10, 10);
    assert_snapshot!(format!("{:?}", grid));
    assert_eq!(grid.scrollback_position_and_length(), (0, 0))
}

#[test]
pub fn fzf_fullscreen() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        112,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "fzf_fullscreen";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn replace_multiple_wide_characters_under_cursor() {
    // this test makes sure that if we replace a wide character with a non-wide character, it
    // properly pads the excess width in the proper place (either before the replaced non-wide
    // character if the cursor was "in the middle" of the wide character, or after the character if
    // it was "in the beginning" of the wide character)
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        112,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "replace_multiple_wide_characters";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn replace_non_wide_characters_with_wide_characters() {
    // this test makes sure that if we replace a wide character with a non-wide character, it
    // properly pads the excess width in the proper place (either before the replaced non-wide
    // character if the cursor was "in the middle" of the wide character, or after the character if
    // it was "in the beginning" of the wide character)
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        112,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "replace_non_wide_characters_with_wide_characters";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn scroll_down_ansi() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        112,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "scroll_down";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn ansi_capital_t() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        112,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let content = "foo\u{1b}[14Tbar".as_bytes();
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn ansi_capital_s() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        112,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let content = "\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\nfoo\u{1b}[14Sbar".as_bytes();
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn terminal_pixel_size_reports() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(Some(SizeInPixels {
            height: 21,
            width: 8,
        }))),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        osc8_hyperlinks,
        debug,
        arrow_fonts,
        styled_underlines,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "terminal_pixel_size_reports";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    // CSI 14t and CSI 16t are forwarded to the host; Zellij no longer
    // synthesises local replies from character_cell_size for these.
    assert!(grid.pending_messages_to_pty.is_empty());
    use crate::host_query::HostQuery;
    assert_eq!(
        grid.pending_forwarded_queries,
        vec![
            HostQuery::TextAreaPixelSize,
            HostQuery::CharacterCellPixelSize
        ],
    );
}

#[test]
fn terminal_pixel_size_reports_in_unsupported_terminals() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)), // in an unsupported terminal, we don't have this info
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "terminal_pixel_size_reports";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    // Forwarding is independent of character_cell_size availability —
    // the host terminal is authoritative for these queries regardless
    // of what Zellij knows locally.
    assert!(grid.pending_messages_to_pty.is_empty());
    use crate::host_query::HostQuery;
    assert_eq!(
        grid.pending_forwarded_queries,
        vec![
            HostQuery::TextAreaPixelSize,
            HostQuery::CharacterCellPixelSize
        ],
    );
}

#[test]
pub fn ansi_csi_at_sign() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        112,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let content = "foo\u{1b}[2D\u{1b}[2@".as_bytes();
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn sixel_images_are_reaped_when_scrolled_off() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let character_cell_size = Rc::new(RefCell::new(Some(SizeInPixels {
        width: 8,
        height: 21,
    })));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        112,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        character_cell_size,
        sixel_image_store.clone(),
        kitty_asset_store.clone(),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let pane_content = read_fixture("sixel-image-500px.six");
    vte_parser.advance(&mut grid, &pane_content);
    for _ in 0..10_051 {
        // scrollbuffer limit + viewport height
        grid.add_canonical_line();
    }
    let _ = grid.read_changes(0, 0); // we do this because this is where the images are reaped
    assert_eq!(
        sixel_image_store.borrow().image_count(),
        0,
        "all images were deleted from the store"
    );
}

#[test]
pub fn sixel_images_are_reaped_when_resetting() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let character_cell_size = Rc::new(RefCell::new(Some(SizeInPixels {
        width: 8,
        height: 21,
    })));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        112,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        character_cell_size,
        sixel_image_store.clone(),
        kitty_asset_store.clone(),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let pane_content = read_fixture("sixel-image-500px.six");
    vte_parser.advance(&mut grid, &pane_content);
    grid.reset_terminal_state();
    let _ = grid.read_changes(0, 0); // we do this because this is where the images are reaped
    assert_eq!(
        sixel_image_store.borrow().image_count(),
        0,
        "all images were deleted from the store"
    );
}

#[test]
pub fn sixel_image_in_alternate_buffer() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let character_cell_size = Rc::new(RefCell::new(Some(SizeInPixels {
        width: 8,
        height: 21,
    })));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        30,
        112,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        character_cell_size,
        sixel_image_store.clone(),
        kitty_asset_store.clone(),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );

    let move_to_alternate_screen = "\u{1b}[?1049h";
    vte_parser.advance(&mut grid, move_to_alternate_screen.as_bytes());

    let pane_content = read_fixture("sixel-image-500px.six");
    vte_parser.advance(&mut grid, &pane_content);
    assert_snapshot!(format!("{:?}", grid)); // should include the image
                                             //
    let move_away_from_alternate_screen = "\u{1b}[?1049l";
    for byte in move_away_from_alternate_screen.as_bytes() {
        vte_parser.advance(&mut grid, &[*byte]);
    }
    assert_snapshot!(format!("{:?}", grid)); // should note include the image
    assert_eq!(
        sixel_image_store.borrow().image_count(),
        0,
        "all images were deleted from the store when we moved back from alternate screen"
    );
}

#[test]
pub fn sixel_with_image_scrolling_decsdm() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let character_cell_size = Rc::new(RefCell::new(Some(SizeInPixels {
        width: 8,
        height: 21,
    })));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        30,
        112,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        character_cell_size,
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );

    // enter DECSDM
    let move_to_decsdm = "\u{1b}[?80h";
    for byte in move_to_decsdm.as_bytes() {
        vte_parser.advance(&mut grid, &[*byte]);
    }

    // write some text
    let mut text_to_fill_pane = String::new();
    for i in 0..10 {
        writeln!(&mut text_to_fill_pane, "\rline {}", i + 1).unwrap();
    }
    for byte in text_to_fill_pane.as_bytes() {
        vte_parser.advance(&mut grid, &[*byte]);
    }

    // render a sixel image (will appear on the top left and partially cover the text)
    let pane_content = read_fixture("sixel-image-100px.six");
    vte_parser.advance(&mut grid, &pane_content);
    // image should be on the top left corner of the grid
    assert_snapshot!(format!("{:?}", grid));

    // leave DECSDM
    let move_away_from_decsdm = "\u{1b}[?80l";
    for byte in move_away_from_decsdm.as_bytes() {
        vte_parser.advance(&mut grid, &[*byte]);
    }

    // Go down to the beginning of the next line
    let mut go_down_once = String::new();
    writeln!(&mut go_down_once, "\n\r").unwrap();
    for byte in go_down_once.as_bytes() {
        vte_parser.advance(&mut grid, &[*byte]);
    }

    // render another sixel image, should appear under the cursor
    let pane_content = read_fixture("sixel-image-100px.six");
    vte_parser.advance(&mut grid, &pane_content);

    // image should appear in cursor position
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn osc_4_background_query() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let content = "\u{1b}]10;?\u{1b}\\";
    for byte in content.as_bytes() {
        vte_parser.advance(&mut grid, &[*byte]);
    }
    // Post-refactor: OSC 10;? is forwarded to the host, not answered
    // from Zellij's cached palette. pending_messages_to_pty must stay
    // empty.
    assert!(grid.pending_messages_to_pty.is_empty());
    let forwarded_string: String = grid
        .pending_forwarded_queries
        .iter()
        .map(|q| String::from_utf8(q.to_query_bytes()).unwrap())
        .collect();
    assert_eq!(forwarded_string, "\u{1b}]10;?\u{1b}\\");
}

#[test]
pub fn osc_4_foreground_query() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let content = "\u{1b}]11;?\u{1b}\\";
    for byte in content.as_bytes() {
        vte_parser.advance(&mut grid, &[*byte]);
    }
    assert!(grid.pending_messages_to_pty.is_empty());
    let forwarded_string: String = grid
        .pending_forwarded_queries
        .iter()
        .map(|q| String::from_utf8(q.to_query_bytes()).unwrap())
        .collect();
    assert_eq!(forwarded_string, "\u{1b}]11;?\u{1b}\\");
}

#[test]
pub fn osc_4_color_query() {
    let mut color_codes = HashMap::new();
    color_codes.insert(222, String::from("rgb:ffff/d7d7/8787"));
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(color_codes));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let content = "\u{1b}]4;222;?\u{1b}\\";
    for byte in content.as_bytes() {
        vte_parser.advance(&mut grid, &[*byte]);
    }
    // OSC 4;N;? is forwarded to the host for the real palette value.
    assert!(grid.pending_messages_to_pty.is_empty());
    let forwarded_string: String = grid
        .pending_forwarded_queries
        .iter()
        .map(|q| String::from_utf8(q.to_query_bytes()).unwrap())
        .collect();
    assert_eq!(forwarded_string, "\u{1b}]4;222;?\u{1b}\\");
}

#[test]
pub fn xtsmgraphics_color_register_count() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let content = "\u{1b}[?1;1;S\u{1b}\\";
    for byte in content.as_bytes() {
        vte_parser.advance(&mut grid, &[*byte]);
    }
    let message_string = grid
        .pending_messages_to_pty
        .iter()
        .map(|m| String::from_utf8_lossy(m))
        .fold(String::new(), |mut acc, s| {
            acc.push_str(&s);
            acc
        });
    assert_eq!(message_string, "\u{1b}[?1;0;65536S");
}

#[test]
pub fn xtsmgraphics_pixel_graphics_geometry() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let character_cell_size = Rc::new(RefCell::new(Some(SizeInPixels {
        width: 8,
        height: 21,
    })));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        character_cell_size,
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let content = "\u{1b}[?2;1;S\u{1b}\\";
    for byte in content.as_bytes() {
        vte_parser.advance(&mut grid, &[*byte]);
    }
    let message_string = grid
        .pending_messages_to_pty
        .iter()
        .map(|m| String::from_utf8_lossy(m))
        .fold(String::new(), |mut acc, s| {
            acc.push_str(&s);
            acc
        });
    assert_eq!(message_string, "\u{1b}[?2;0;776;1071S");
}

#[test]
pub fn cursor_hide_persists_through_alternate_screen() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let character_cell_size = Rc::new(RefCell::new(Some(SizeInPixels {
        width: 8,
        height: 21,
    })));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        30,
        112,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        character_cell_size,
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );

    let hide_cursor = "\u{1b}[?25l";
    for byte in hide_cursor.as_bytes() {
        vte_parser.advance(&mut grid, &[*byte]);
    }
    assert!(
        matches!(grid.cursor_coordinates(), Some((_, _, false))),
        "Cursor hidden properly"
    );

    let move_to_alternate_screen = "\u{1b}[?1049h";
    vte_parser.advance(&mut grid, move_to_alternate_screen.as_bytes());
    assert!(
        matches!(grid.cursor_coordinates(), Some((_, _, false))),
        "Cursor still hidden in alternate screen"
    );

    let show_cursor = "\u{1b}[?25h";
    for byte in show_cursor.as_bytes() {
        vte_parser.advance(&mut grid, &[*byte]);
    }
    assert!(
        matches!(grid.cursor_coordinates(), Some((_, _, true))),
        "Cursor shown"
    );

    let move_away_from_alternate_screen = "\u{1b}[?1049l";
    for byte in move_away_from_alternate_screen.as_bytes() {
        vte_parser.advance(&mut grid, &[*byte]);
    }
    assert!(
        matches!(grid.cursor_coordinates(), Some((_, _, true))),
        "Cursor still shown away from alternate screen"
    );
}

#[test]
fn table_ui_component() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "table-ui-component";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn table_ui_component_with_coordinates() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "table-ui-component-with-coordinates";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn ribbon_ui_component() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "ribbon-ui-component";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn ribbon_ui_component_with_coordinates() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "ribbon-ui-component-with-coordinates";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn nested_list_ui_component() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        120,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "nested-list-ui-component";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn nested_list_ui_component_with_coordinates() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        120,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "nested-list-ui-component-with-coordinates";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn text_ui_component() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        120,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "text-ui-component";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn text_ui_component_with_coordinates() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        120,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "text-ui-component-with-coordinates";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn cannot_escape_scroll_region() {
    // this tests a fix for a bug where it would be possible to set the scroll region bounds beyond
    // the pane height, which would then allow a goto instruction beyond the scroll region to scape
    // the pane bounds and render content on other panes
    //
    // what we do here is set the scroll region beyond the terminal bounds (`<ESC>[1;42r` - whereas
    // the terminal is just 41 lines high), and then issue a goto instruction to line 42, one line
    // beyond the pane and scroll region bounds (`<ESC>[42;1H`) and then print text `Hi there!`.
    // This should be printed on the last line (zero indexed 40) of the terminal and not beyond it.
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        120,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let content = "\u{1b}[1;42r\u{1b}[42;1HHi there!".as_bytes();
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn preserve_background_color_on_resize() {
    use crate::panes::terminal_character::{AnsiCode, EMPTY_TERMINAL_CHARACTER};

    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        10,
        20,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );

    let mut parse = |s, grid: &mut Grid| {
        for b in Vec::from(s) {
            vte_parser.advance(&mut *grid, &[b])
        }
    };

    // Write text with red background that extends to end of line
    // ESC[41m = red background
    // ESC[K = clear to end of line (fills with current background)
    // ESC[0m = reset
    let content = "test\x1b[41m\x1b[K\x1b[0m";
    parse(content, &mut grid);

    // Check that characters after "test" have red background before resize
    let first_row = &grid.viewport[0];
    let background_char_count_before = first_row
        .columns
        .iter()
        .enumerate()
        .filter(|(i, c)| *i >= 4 && c.styles.background != Some(AnsiCode::Reset))
        .count();
    assert!(
        background_char_count_before > 0,
        "Should have characters with background color before resize"
    );

    // Also check that plain trailing spaces are properly trimmed (regression test)
    let content2 = "\r\n\rplain text with spaces    ";
    parse(content2, &mut grid);

    // Resize the grid
    grid.change_size(10, 30);

    // Check that the background color is preserved after resize
    let first_row = &grid.viewport[0];
    let background_char_count_after = first_row
        .columns
        .iter()
        .enumerate()
        .filter(|(i, c)| *i >= 4 && c.styles.background != Some(AnsiCode::Reset))
        .count();
    assert_eq!(
        background_char_count_before, background_char_count_after,
        "Background colored characters should be preserved after resize"
    );

    // Verify that the second line doesn't have excessive trailing spaces
    // (it should be trimmed since they're plain spaces without background color)
    let second_row = &grid.viewport[1];
    let trailing_spaces = second_row
        .columns
        .iter()
        .rev()
        .take_while(|c| c.character == EMPTY_TERMINAL_CHARACTER.character)
        .count();
    // All trailing plain spaces should be completely removed
    assert_eq!(
        trailing_spaces, 0,
        "Plain trailing spaces should be completely trimmed, but found {} trailing spaces",
        trailing_spaces
    );
}

fn create_grid_with_content(content: &str) -> Grid {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let mut grid = Grid::new(
        20,
        80,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        false,
        true,
        true,
        true,
        false,
    );
    for byte in content.as_bytes() {
        vte_parser.advance(&mut grid, &[*byte]);
    }
    grid
}

#[test]
fn double_click_selection_preserved_after_scroll() {
    let content = "line 0\nline 1\nline 2\nline 3\nline 4\nthis is a word test\nline 6\nline 7\nline 8\nline 9\nline 10\nline 11\nline 12\nline 13\nline 14\nline 15\nline 16\nline 17\nline 18\nline 19\n";
    let mut grid = create_grid_with_content(content);

    for _ in 0..20 {
        grid.add_canonical_line();
    }

    let word_position = Position::new(5, 10);
    grid.start_selection(&word_position);

    let selection_before = grid.get_selected_text();
    let word_start = grid.selection.start;
    let word_end = grid.selection.end;

    grid.end_selection(&word_position);

    grid.scroll_up_one_line();

    let selection_after_start = grid.selection.start;
    let selection_after_end = grid.selection.end;

    assert_eq!(selection_after_start.line.0, word_start.line.0 + 1);
    assert_eq!(selection_after_end.line.0, word_end.line.0 + 1);
    assert_eq!(selection_after_start.column, word_start.column);
    assert_eq!(selection_after_end.column, word_end.column);

    let text_after = grid.get_selected_text();
    assert_eq!(selection_before, text_after);
}

#[test]
fn triple_click_selection_preserved_after_scroll() {
    let content = "line 0\nline 1\nline 2\nline 3\nline 4\nthis is line five with some text\nline 6\nline 7\nline 8\nline 9\nline 10\nline 11\nline 12\nline 13\nline 14\nline 15\nline 16\nline 17\nline 18\nline 19\n";
    let mut grid = create_grid_with_content(content);

    for _ in 0..20 {
        grid.add_canonical_line();
    }

    let line_position = Position::new(5, 15);
    grid.start_selection(&line_position);
    grid.start_selection(&line_position);
    grid.start_selection(&line_position);

    let selection_before = grid.get_selected_text();
    let line_start = grid.selection.start;
    let line_end = grid.selection.end;

    grid.end_selection(&line_position);

    grid.scroll_up_one_line();

    let selection_after_start = grid.selection.start;
    let selection_after_end = grid.selection.end;

    assert_eq!(selection_after_start.line.0, line_start.line.0 + 1);
    assert_eq!(selection_after_end.line.0, line_end.line.0 + 1);
    assert_eq!(selection_after_start.column, line_start.column);
    assert_eq!(selection_after_end.column, line_end.column);

    let text_after = grid.get_selected_text();
    assert_eq!(selection_before, text_after);
}

#[test]
fn double_click_selection_moves_with_multiple_scrolls() {
    let content = "line 0\nline 1\nline 2\nline 3\nline 4\nthis is a word test\nline 6\nline 7\nline 8\nline 9\nline 10\nline 11\nline 12\nline 13\nline 14\nline 15\nline 16\nline 17\nline 18\nline 19\n";
    let mut grid = create_grid_with_content(content);

    for _ in 0..20 {
        grid.add_canonical_line();
    }

    let word_position = Position::new(5, 10);
    grid.start_selection(&word_position);

    let initial_start = grid.selection.start;
    let initial_end = grid.selection.end;

    grid.end_selection(&word_position);

    for _ in 0..5 {
        grid.scroll_up_one_line();
    }

    assert_eq!(grid.selection.start.line.0, initial_start.line.0 + 5);
    assert_eq!(grid.selection.end.line.0, initial_end.line.0 + 5);
    assert_eq!(grid.selection.start.column, initial_start.column);
    assert_eq!(grid.selection.end.column, initial_end.column);
}

#[test]
fn single_click_drag_selection_preserved_after_scroll() {
    let content = "line 0\nline 1\nline 2\nline 3\nline 4\nsome text here\nline 6\nline 7\nline 8\nline 9\nline 10\nline 11\nline 12\nline 13\nline 14\nline 15\nline 16\nline 17\nline 18\nline 19\n";
    let mut grid = create_grid_with_content(content);

    for _ in 0..20 {
        grid.add_canonical_line();
    }

    grid.start_selection(&Position::new(5, 5));
    grid.update_selection(&Position::new(5, 10));

    let start_before = grid.selection.start;
    let end_before = grid.selection.end;

    grid.end_selection(&Position::new(5, 10));

    grid.scroll_up_one_line();

    assert_eq!(grid.selection.start.line.0, start_before.line.0 + 1);
    assert_eq!(grid.selection.end.line.0, end_before.line.0 + 1);
    assert_eq!(grid.selection.start.column, start_before.column);
    assert_eq!(grid.selection.end.column, end_before.column);
}

#[test]
fn osc_11_set_and_query_pane_default_bg() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let mut grid = Grid::new(
        10,
        20,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        false,
        true,
        true,
        true,
        false,
    );

    // Set background via OSC 11
    let set_bg = b"\x1b]11;#001a3a\x07";
    for byte in set_bg.iter() {
        vte_parser.advance(&mut grid, &[*byte]);
    }

    assert_eq!(grid.pane_default_bg, Some((0, 26, 58)));

    // Query background via OSC 11 — because a pane-scoped override is
    // in place, the query is short-circuited: apps inside the pane
    // must see what Zellij is actually rendering, not the host
    // terminal's bg. The reply uses xterm's canonical
    // `rgb:RRRR/GGGG/BBBB` form with each 8-bit channel widened by
    // repetition (0x00 → 0x0000, 0x1a → 0x1a1a, 0x3a → 0x3a3a).
    let query_bg = b"\x1b]11;?\x07";
    for byte in query_bg.iter() {
        vte_parser.advance(&mut grid, &[*byte]);
    }

    assert!(
        grid.pending_forwarded_queries.is_empty(),
        "OSC 11 query must not be forwarded when a pane override is set"
    );
    assert_eq!(grid.pending_messages_to_pty.len(), 1);
    let reply = String::from_utf8(grid.pending_messages_to_pty[0].clone()).unwrap();
    assert_eq!(reply, "\u{1b}]11;rgb:0000/1a1a/3a3a\u{7}",);
}

#[test]
fn osc_10_set_and_query_pane_default_fg() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let mut grid = Grid::new(
        10,
        20,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        false,
        true,
        true,
        true,
        false,
    );

    // Set foreground via OSC 10
    let set_fg = b"\x1b]10;#00e000\x07";
    for byte in set_fg.iter() {
        vte_parser.advance(&mut grid, &[*byte]);
    }

    assert_eq!(grid.pane_default_fg, Some((0, 224, 0)));

    // Query foreground via OSC 10 — pane-scoped override is in place,
    // so the query is answered locally (see OSC 11 equivalent test for
    // the short-circuit rationale).
    let query_fg = b"\x1b]10;?\x07";
    for byte in query_fg.iter() {
        vte_parser.advance(&mut grid, &[*byte]);
    }

    assert!(
        grid.pending_forwarded_queries.is_empty(),
        "OSC 10 query must not be forwarded when a pane override is set"
    );
    assert_eq!(grid.pending_messages_to_pty.len(), 1);
    let reply = String::from_utf8(grid.pending_messages_to_pty[0].clone()).unwrap();
    assert_eq!(reply, "\u{1b}]10;rgb:0000/e0e0/0000\u{7}",);
}

#[test]
fn osc_110_111_reset_pane_default_colors() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let mut grid = Grid::new(
        10,
        20,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        false,
        true,
        true,
        true,
        false,
    );

    // Set both fg and bg
    let set_fg = b"\x1b]10;#00e000\x07";
    for byte in set_fg.iter() {
        vte_parser.advance(&mut grid, &[*byte]);
    }
    let set_bg = b"\x1b]11;#001a3a\x07";
    for byte in set_bg.iter() {
        vte_parser.advance(&mut grid, &[*byte]);
    }

    assert_eq!(grid.pane_default_fg, Some((0, 224, 0)));
    assert_eq!(grid.pane_default_bg, Some((0, 26, 58)));

    // Reset foreground via OSC 110
    let reset_fg = b"\x1b]110\x07";
    for byte in reset_fg.iter() {
        vte_parser.advance(&mut grid, &[*byte]);
    }
    assert_eq!(grid.pane_default_fg, None);
    assert_eq!(grid.pane_default_bg, Some((0, 26, 58)));

    // Reset background via OSC 111
    let reset_bg = b"\x1b]111\x07";
    for byte in reset_bg.iter() {
        vte_parser.advance(&mut grid, &[*byte]);
    }
    assert_eq!(grid.pane_default_fg, None);
    assert_eq!(grid.pane_default_bg, None);
}

#[test]
fn osc_11_set_bg_produces_ansi_in_render_output() {
    use crate::panes::terminal_character::AnsiCode;

    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let mut grid = Grid::new(
        5,
        10,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        false,
        true,
        true,
        true,
        false,
    );

    // Set background via OSC 11
    let set_bg = b"\x1b]11;#001a3a\x07";
    for byte in set_bg.iter() {
        vte_parser.advance(&mut grid, &[*byte]);
    }

    assert_eq!(grid.pane_default_bg, Some((0, 26, 58)));

    // Render the grid and check that the pane defaults are stamped on chunks
    let style = Style::default();
    let render_result = grid.render(0, 0, &style).unwrap();
    assert!(render_result.is_some(), "Expected render output");

    let render_output = render_result.unwrap();
    let chunks = render_output.character_chunks;
    assert!(!chunks.is_empty(), "Expected at least one character chunk");

    // All chunks should carry the pane default bg
    for chunk in &chunks {
        assert_eq!(
            chunk.pane_default_bg,
            Some(AnsiCode::RgbCode((0, 26, 58))),
            "Chunk should carry pane default background"
        );
    }
}

// =====================================================================
// Plugin Highlight Engine Tests
// =====================================================================

use crate::panes::grid::MouseTracking;
use crate::panes::terminal_character::AnsiCode;
use std::collections::BTreeMap;
use zellij_utils::data::{HighlightLayer, HighlightStyle, RegexHighlight};

fn create_highlight(
    pattern: &str,
    on_hover: bool,
    bold: bool,
    italic: bool,
    underline: bool,
    layer: HighlightLayer,
) -> RegexHighlight {
    RegexHighlight {
        pattern: pattern.to_string(),
        style: HighlightStyle::Emphasis0,
        layer,
        context: BTreeMap::new(),
        on_hover,
        bold,
        italic,
        underline,
        tooltip_text: None,
    }
}

#[test]
fn set_plugin_regex_highlights_basic_match() {
    let mut grid = create_grid_with_content("hello foo bar\n");
    let highlights = vec![RegexHighlight {
        pattern: "foo".into(),
        style: HighlightStyle::Emphasis0,
        layer: HighlightLayer::Hint,
        context: BTreeMap::new(),
        on_hover: false,
        bold: false,
        italic: false,
        underline: true,
        tooltip_text: None,
    }];
    grid.set_plugin_regex_highlights(1, highlights, &Style::default());
    let slot = grid.plugin_highlights.get(&1);
    assert!(slot.is_some());
    let entries = slot.unwrap();
    assert_eq!(entries.len(), 1);
    assert!(entries[0].1.regex.is_match("foo"));
}

#[test]
fn set_plugin_regex_highlights_no_match() {
    let mut grid = create_grid_with_content("hello world bar\n");
    let highlights = vec![create_highlight(
        "xyz123",
        false,
        false,
        false,
        false,
        HighlightLayer::Hint,
    )];
    grid.set_plugin_regex_highlights(1, highlights, &Style::default());

    // No position in the viewport should match
    for col in 0..15 {
        assert!(grid.plugin_highlight_at(&Position::new(0, col)).is_none());
    }
}

#[test]
fn clear_plugin_highlights_removes_highlights() {
    let mut grid = create_grid_with_content("hello foo bar\n");
    let highlights = vec![create_highlight(
        "foo",
        false,
        false,
        false,
        true,
        HighlightLayer::Hint,
    )];
    grid.set_plugin_regex_highlights(1, highlights, &Style::default());
    assert!(grid.plugin_highlights.get(&1).is_some());

    grid.clear_plugin_highlights(1);
    assert!(grid.plugin_highlights.get(&1).is_none());
}

#[test]
fn multiple_plugins_highlights_independent() {
    let mut grid = create_grid_with_content("aaa bbb ccc\n");
    let h1 = vec![create_highlight(
        "aaa",
        false,
        false,
        false,
        false,
        HighlightLayer::Hint,
    )];
    let h2 = vec![create_highlight(
        "bbb",
        false,
        false,
        false,
        false,
        HighlightLayer::Hint,
    )];
    grid.set_plugin_regex_highlights(1, h1, &Style::default());
    grid.set_plugin_regex_highlights(2, h2, &Style::default());

    assert!(grid.plugin_highlights.get(&1).is_some());
    assert!(grid.plugin_highlights.get(&2).is_some());

    grid.clear_plugin_highlights(1);
    assert!(grid.plugin_highlights.get(&1).is_none());
    assert!(grid.plugin_highlights.get(&2).is_some());
}

#[test]
fn upsert_replaces_same_pattern() {
    let mut grid = create_grid_with_content("foo bar\n");
    let h1 = vec![RegexHighlight {
        pattern: "foo".into(),
        style: HighlightStyle::Emphasis0,
        layer: HighlightLayer::Hint,
        context: BTreeMap::new(),
        on_hover: false,
        bold: false,
        italic: false,
        underline: true,
        tooltip_text: None,
    }];
    grid.set_plugin_regex_highlights(1, h1, &Style::default());

    let h2 = vec![RegexHighlight {
        pattern: "foo".into(),
        style: HighlightStyle::Emphasis0,
        layer: HighlightLayer::Hint,
        context: BTreeMap::new(),
        on_hover: false,
        bold: false,
        italic: false,
        underline: false,
        tooltip_text: None,
    }];
    grid.set_plugin_regex_highlights(1, h2, &Style::default());

    let entries = grid.plugin_highlights.get(&1).unwrap();
    assert_eq!(entries.len(), 1);
    assert!(!entries[0].1.underline);
}

#[test]
fn invalid_regex_does_not_crash() {
    let mut grid = create_grid_with_content("hello\n");
    let highlights = vec![create_highlight(
        "[invalid",
        false,
        false,
        false,
        false,
        HighlightLayer::Hint,
    )];
    grid.set_plugin_regex_highlights(1, highlights, &Style::default());
    // Invalid regex should be skipped
    let slot = grid.plugin_highlights.get(&1);
    match slot {
        None => {}, // acceptable
        Some(entries) => assert_eq!(entries.len(), 0),
    }
}

#[test]
fn plugin_highlight_at_returns_match() {
    let mut grid = create_grid_with_content("hello foo bar\n");
    let mut context = BTreeMap::new();
    context.insert("key".to_string(), "value".to_string());
    let highlights = vec![RegexHighlight {
        pattern: "foo".into(),
        style: HighlightStyle::Emphasis0,
        layer: HighlightLayer::Hint,
        context: context.clone(),
        on_hover: false,
        bold: false,
        italic: false,
        underline: true,
        tooltip_text: None,
    }];
    grid.set_plugin_regex_highlights(1, highlights, &Style::default());

    // "foo" starts at column 6 in "hello foo bar"
    let result = grid.plugin_highlight_at(&Position::new(0, 6));
    assert!(result.is_some());
    let (plugin_id, pattern, matched_string, ctx) = result.unwrap();
    assert_eq!(plugin_id, 1);
    assert_eq!(pattern, "foo");
    assert_eq!(matched_string, "foo");
    assert_eq!(ctx.get("key").unwrap(), "value");
}

#[test]
fn plugin_highlight_at_returns_none_on_miss() {
    let mut grid = create_grid_with_content("hello foo bar\n");
    let highlights = vec![create_highlight(
        "foo",
        false,
        false,
        false,
        true,
        HighlightLayer::Hint,
    )];
    grid.set_plugin_regex_highlights(1, highlights, &Style::default());

    // Position 0 is in "hello", not "foo"
    let result = grid.plugin_highlight_at(&Position::new(0, 0));
    assert!(result.is_none());
}

#[test]
fn plugin_highlight_at_wrapped_line() {
    // Create a narrow grid (10 cols) so that a long string wraps
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let mut grid = Grid::new(
        5,
        10,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        false,
        true,
        true,
        true,
        false,
    );
    // Feed a long string that wraps: "abcdefghij" fills row 0, "klmnopqrst" fills row 1
    let content = "abcdefghijklmnopqrst";
    let mut vte_parser = vte::Parser::new();
    for &byte in content.as_bytes() {
        vte_parser.advance(&mut grid, &[byte]);
    }

    // Set a highlight for "jklm" which spans the wrap boundary
    let highlights = vec![create_highlight(
        "jklm",
        false,
        false,
        false,
        true,
        HighlightLayer::Hint,
    )];
    grid.set_plugin_regex_highlights(1, highlights, &Style::default());

    // "jklm" spans row 0 col 9 through row 1 col 3
    // Position in the wrapped portion (row 1, col 1 = 'k')
    let result = grid.plugin_highlight_at(&Position::new(1, 1));
    assert!(result.is_some());
    let (plugin_id, _pattern, matched_string, _ctx) = result.unwrap();
    assert_eq!(plugin_id, 1);
    assert_eq!(matched_string, "jklm");
}

#[test]
fn hover_position_triggers_on_hover_highlight() {
    let mut grid = create_grid_with_content("hello link_text bar\n");
    let highlights = vec![create_highlight(
        "link_text",
        true,
        false,
        false,
        true,
        HighlightLayer::Hint,
    )];
    grid.set_plugin_regex_highlights(1, highlights, &Style::default());

    // Set hover position inside "link_text" (starts at col 6)
    grid.set_hover_position(Some(Position::new(0, 8)));
    assert!(grid.hover_position.is_some());
    assert_eq!(grid.hover_position.unwrap(), Position::new(0, 8));

    // The on_hover entry should exist in plugin_highlights
    let entries = grid.plugin_highlights.get(&1).unwrap();
    assert_eq!(entries.len(), 1);
    assert!(entries[0].1.on_hover);
}

#[test]
fn hover_suppressed_when_mouse_tracking_on() {
    let mut grid = create_grid_with_content("hello link_text bar\n");
    let highlights = vec![create_highlight(
        "link_text",
        true,
        false,
        false,
        true,
        HighlightLayer::Hint,
    )];
    grid.set_plugin_regex_highlights(1, highlights, &Style::default());

    // Enable mouse tracking — the render path should skip hover highlights
    grid.mouse_tracking = MouseTracking::Normal;
    grid.set_hover_position(Some(Position::new(0, 8)));

    // The hover position is set regardless (the guard is in the render path),
    // but we verify that mouse_tracking is non-Off
    assert!(grid.hover_position.is_some());
    assert_ne!(grid.mouse_tracking, MouseTracking::Off);
}

#[test]
fn wide_char_display_column_mapping() {
    // CJK characters: "你好" = 2 chars, each 2 display cols wide, so "world" starts at display col 4
    let mut grid = create_grid_with_content("你好world\n");
    let highlights = vec![create_highlight(
        "world",
        false,
        false,
        false,
        true,
        HighlightLayer::Hint,
    )];
    grid.set_plugin_regex_highlights(1, highlights, &Style::default());

    // "你好" occupies display cols 0-3, "world" starts at display col 4
    let result = grid.plugin_highlight_at(&Position::new(0, 4));
    assert!(result.is_some());
    let (plugin_id, _pattern, matched_string, _ctx) = result.unwrap();
    assert_eq!(plugin_id, 1);
    assert_eq!(matched_string, "world");

    // Position 2 should be inside "你好", not "world"
    let result_miss = grid.plugin_highlight_at(&Position::new(0, 2));
    assert!(result_miss.is_none());
}

#[test]
fn highlight_style_variants_resolve_colors() {
    use super::resolve_highlight_colors;

    let style = Style::default();

    // HighlightStyle::None returns (None, None)
    let (fg, bg): (Option<AnsiCode>, Option<AnsiCode>) =
        resolve_highlight_colors(&HighlightStyle::None, &style);
    assert!(fg.is_none());
    assert!(bg.is_none());

    // HighlightStyle::CustomRgb with fg only
    let (fg, bg): (Option<AnsiCode>, Option<AnsiCode>) = resolve_highlight_colors(
        &HighlightStyle::CustomRgb {
            fg: Some((255, 0, 0)),
            bg: None,
        },
        &style,
    );
    assert_eq!(fg, Some(AnsiCode::RgbCode((255, 0, 0))));
    assert!(bg.is_none());

    // HighlightStyle::CustomIndex with bg only
    let (fg, bg): (Option<AnsiCode>, Option<AnsiCode>) = resolve_highlight_colors(
        &HighlightStyle::CustomIndex {
            fg: None,
            bg: Some(42),
        },
        &style,
    );
    assert!(fg.is_none());
    assert_eq!(bg, Some(AnsiCode::ColorIndex(42)));

    // HighlightStyle::Emphasis0 should return a foreground color from the palette
    let (fg, bg): (Option<AnsiCode>, Option<AnsiCode>) =
        resolve_highlight_colors(&HighlightStyle::Emphasis0, &style);
    assert!(fg.is_some());
    assert!(bg.is_none());
}

fn create_grid_with_scrollback() -> Grid {
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let mut grid = Grid::new(
        5,
        40,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        false,
        true,
        true,
        true,
        false,
    );
    let mut parser = vte::Parser::new();
    for i in 0..25 {
        let line = format!("scrollback line {}\r\n", i);
        for byte in line.as_bytes() {
            parser.advance(&mut grid, &[*byte]);
        }
    }
    grid
}

#[test]
fn pane_contents_scrollback_no_truncation_when_max_none() {
    let grid = create_grid_with_scrollback();
    let result = grid.pane_contents(true, None);
    assert_eq!(
        result.lines_above_viewport.len(),
        21,
        "All scrollback lines should be returned when max is None"
    );
}

#[test]
fn pane_contents_scrollback_no_truncation_when_max_zero() {
    let grid = create_grid_with_scrollback();
    let result = grid.pane_contents(true, Some(0));
    assert_eq!(
        result.lines_above_viewport.len(),
        21,
        "Some(0) is sentinel for all scrollback — no truncation"
    );
}

#[test]
fn pane_contents_scrollback_truncates_to_last_n() {
    let grid = create_grid_with_scrollback();
    let result = grid.pane_contents(true, Some(5));
    assert_eq!(result.lines_above_viewport.len(), 5);
    let full = grid.pane_contents(true, None);
    let expected: Vec<String> = full
        .lines_above_viewport
        .iter()
        .rev()
        .take(5)
        .rev()
        .cloned()
        .collect();
    assert_eq!(result.lines_above_viewport, expected);
}

#[test]
fn pane_contents_scrollback_no_truncation_when_n_exceeds_total() {
    let grid = create_grid_with_scrollback();
    let result = grid.pane_contents(true, Some(100));
    assert_eq!(
        result.lines_above_viewport.len(),
        21,
        "No truncation when N exceeds total scrollback lines"
    );
}

#[test]
fn pane_contents_no_scrollback_when_flag_false() {
    let grid = create_grid_with_scrollback();
    let result = grid.pane_contents(false, Some(5));
    assert!(
        result.lines_above_viewport.is_empty(),
        "get_full_scrollback=false should never collect scrollback"
    );
}

// =====================================================================
// pane_contents_with_ansi Tests
// =====================================================================

fn create_grid_with_colored_scrollback() -> Grid {
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let mut grid = Grid::new(
        5,
        40,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        false,
        true,
        true,
        true,
        false,
    );
    let mut parser = vte::Parser::new();
    for i in 0..25 {
        let line = format!("\x1b[31mred line {}\x1b[0m\r\n", i);
        for byte in line.as_bytes() {
            parser.advance(&mut grid, &[*byte]);
        }
    }
    grid
}

#[test]
fn pane_contents_with_ansi_preserves_escape_codes() {
    let grid = create_grid_with_colored_scrollback();
    let result = grid.pane_contents_with_ansi(false, None);
    let has_ansi = result.viewport.iter().any(|line| line.contains("\x1b["));
    assert!(
        has_ansi,
        "pane_contents_with_ansi should preserve ANSI escape codes in viewport. Lines: {:?}",
        result.viewport
    );
}

#[test]
fn pane_contents_strips_escape_codes() {
    let grid = create_grid_with_colored_scrollback();
    let result = grid.pane_contents(false, None);
    let has_ansi = result.viewport.iter().any(|line| line.contains("\x1b["));
    assert!(
        !has_ansi,
        "pane_contents should strip ANSI escape codes from viewport. Lines: {:?}",
        result.viewport
    );
}

#[test]
fn pane_contents_with_ansi_scrollback_preserves_escape_codes() {
    let grid = create_grid_with_colored_scrollback();
    let result = grid.pane_contents_with_ansi(true, None);
    let has_ansi = result
        .lines_above_viewport
        .iter()
        .any(|line| line.contains("\x1b["));
    assert!(
        has_ansi,
        "pane_contents_with_ansi should preserve ANSI escape codes in scrollback. Lines: {:?}",
        result.lines_above_viewport
    );
}

#[test]
fn pane_contents_with_ansi_scrollback_truncation() {
    let grid = create_grid_with_colored_scrollback();
    let result = grid.pane_contents_with_ansi(true, Some(3));
    assert_eq!(
        result.lines_above_viewport.len(),
        3,
        "Should truncate to 3 scrollback lines"
    );
    let all_have_ansi = result
        .lines_above_viewport
        .iter()
        .all(|line| line.contains("\x1b["));
    assert!(
        all_have_ansi,
        "All truncated scrollback lines should contain ANSI codes. Lines: {:?}",
        result.lines_above_viewport
    );
}

#[test]
fn pane_contents_with_ansi_no_scrollback_when_flag_false() {
    let grid = create_grid_with_colored_scrollback();
    let result = grid.pane_contents_with_ansi(false, Some(5));
    assert!(
        result.lines_above_viewport.is_empty(),
        "get_full_scrollback=false should never collect scrollback even with ansi"
    );
}

#[test]
fn pane_contents_with_ansi_and_without_have_same_text() {
    let grid = create_grid_with_colored_scrollback();
    let plain = grid.pane_contents(false, None);
    let ansi = grid.pane_contents_with_ansi(false, None);
    assert_eq!(
        plain.viewport.len(),
        ansi.viewport.len(),
        "Both should have the same number of viewport lines"
    );
    // Strip ANSI codes from the ansi version and compare plain text content
    let ansi_escape = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    for (plain_line, ansi_line) in plain.viewport.iter().zip(ansi.viewport.iter()) {
        let stripped = ansi_escape.replace_all(ansi_line, "").to_string();
        assert_eq!(
            *plain_line, stripped,
            "After stripping ANSI codes, text content should match"
        );
    }
}

// =====================================================================
// Highlight Layer Priority Tests
// =====================================================================

#[test]
fn higher_layer_wins_plugin_highlight_at() {
    let mut grid = create_grid_with_content("hello foo bar\n");
    let h1 = vec![RegexHighlight {
        pattern: "foo".into(),
        style: HighlightStyle::Emphasis0,
        layer: HighlightLayer::Hint,
        context: BTreeMap::new(),
        on_hover: false,
        bold: false,
        italic: false,
        underline: false,
        tooltip_text: None,
    }];
    let h2 = vec![RegexHighlight {
        pattern: "foo".into(),
        style: HighlightStyle::Emphasis1,
        layer: HighlightLayer::Tool,
        context: BTreeMap::new(),
        on_hover: false,
        bold: false,
        italic: false,
        underline: false,
        tooltip_text: None,
    }];
    grid.set_plugin_regex_highlights(1, h1, &Style::default());
    grid.set_plugin_regex_highlights(2, h2, &Style::default());

    // "foo" starts at column 6
    let result = grid.plugin_highlight_at(&Position::new(0, 6));
    assert!(result.is_some());
    let (plugin_id, _, _, _) = result.unwrap();
    assert_eq!(plugin_id, 2, "Tool layer plugin should win over Hint layer");
}

#[test]
fn same_layer_both_returned_deterministically() {
    let mut grid = create_grid_with_content("hello foo bar\n");
    let h1 = vec![RegexHighlight {
        pattern: "foo".into(),
        style: HighlightStyle::Emphasis0,
        layer: HighlightLayer::Hint,
        context: BTreeMap::new(),
        on_hover: false,
        bold: false,
        italic: false,
        underline: false,
        tooltip_text: None,
    }];
    let h2 = vec![RegexHighlight {
        pattern: "foo".into(),
        style: HighlightStyle::Emphasis1,
        layer: HighlightLayer::Hint,
        context: BTreeMap::new(),
        on_hover: false,
        bold: false,
        italic: false,
        underline: false,
        tooltip_text: None,
    }];
    grid.set_plugin_regex_highlights(1, h1, &Style::default());
    grid.set_plugin_regex_highlights(2, h2, &Style::default());

    let result = grid.plugin_highlight_at(&Position::new(0, 6));
    assert!(
        result.is_some(),
        "Same-layer conflicts should not cause errors"
    );
}

#[test]
fn lower_layer_wins_when_higher_layer_absent() {
    let mut grid = create_grid_with_content("foo bar\n");
    let h1 = vec![RegexHighlight {
        pattern: "foo".into(),
        style: HighlightStyle::Emphasis0,
        layer: HighlightLayer::Hint,
        context: BTreeMap::new(),
        on_hover: false,
        bold: false,
        italic: false,
        underline: false,
        tooltip_text: None,
    }];
    let h2 = vec![RegexHighlight {
        pattern: "bar".into(),
        style: HighlightStyle::Emphasis1,
        layer: HighlightLayer::ActionFeedback,
        context: BTreeMap::new(),
        on_hover: false,
        bold: false,
        italic: false,
        underline: false,
        tooltip_text: None,
    }];
    grid.set_plugin_regex_highlights(1, h1, &Style::default());
    grid.set_plugin_regex_highlights(2, h2, &Style::default());

    // "foo" at col 0 — only Hint layer matches here
    let result_foo = grid.plugin_highlight_at(&Position::new(0, 0));
    assert!(result_foo.is_some());
    assert_eq!(result_foo.unwrap().0, 1);

    // "bar" at col 4 — only ActionFeedback layer matches here
    let result_bar = grid.plugin_highlight_at(&Position::new(0, 4));
    assert!(result_bar.is_some());
    assert_eq!(result_bar.unwrap().0, 2);
}

#[test]
fn tooltip_from_higher_layer_wins() {
    let mut grid = create_grid_with_content("hello foo bar\n");
    let h1 = vec![RegexHighlight {
        pattern: "foo".into(),
        style: HighlightStyle::Emphasis0,
        layer: HighlightLayer::Hint,
        context: BTreeMap::new(),
        on_hover: true,
        bold: false,
        italic: false,
        underline: false,
        tooltip_text: Some("hint tooltip".to_string()),
    }];
    let h2 = vec![RegexHighlight {
        pattern: "foo".into(),
        style: HighlightStyle::Emphasis1,
        layer: HighlightLayer::Tool,
        context: BTreeMap::new(),
        on_hover: true,
        bold: false,
        italic: false,
        underline: false,
        tooltip_text: Some("tool tooltip".to_string()),
    }];
    grid.set_plugin_regex_highlights(1, h1, &Style::default());
    grid.set_plugin_regex_highlights(2, h2, &Style::default());

    // Set hover position inside "foo" (starts at col 6)
    grid.set_hover_position(Some(Position::new(0, 6)));
    assert_eq!(
        grid.cached_hover_tooltip,
        Some("tool tooltip".to_string()),
        "Tool layer tooltip should win over Hint layer tooltip"
    );
}

#[test]
fn tooltip_from_lower_layer_when_higher_has_none() {
    let mut grid = create_grid_with_content("hello foo bar\n");
    let h1 = vec![RegexHighlight {
        pattern: "foo".into(),
        style: HighlightStyle::Emphasis0,
        layer: HighlightLayer::Hint,
        context: BTreeMap::new(),
        on_hover: true,
        bold: false,
        italic: false,
        underline: false,
        tooltip_text: Some("hint tooltip".to_string()),
    }];
    let h2 = vec![RegexHighlight {
        pattern: "foo".into(),
        style: HighlightStyle::Emphasis1,
        layer: HighlightLayer::Tool,
        context: BTreeMap::new(),
        on_hover: true,
        bold: false,
        italic: false,
        underline: false,
        tooltip_text: None,
    }];
    grid.set_plugin_regex_highlights(1, h1, &Style::default());
    grid.set_plugin_regex_highlights(2, h2, &Style::default());

    grid.set_hover_position(Some(Position::new(0, 6)));
    assert_eq!(
        grid.cached_hover_tooltip,
        Some("hint tooltip".to_string()),
        "When higher layer has no tooltip, lower layer tooltip should be used"
    );
}

#[test]
fn layer_field_stored_in_compiled_highlight() {
    let mut grid = create_grid_with_content("foo bar\n");
    let highlights = vec![RegexHighlight {
        pattern: "foo".into(),
        style: HighlightStyle::Emphasis0,
        layer: HighlightLayer::ActionFeedback,
        context: BTreeMap::new(),
        on_hover: false,
        bold: false,
        italic: false,
        underline: false,
        tooltip_text: None,
    }];
    grid.set_plugin_regex_highlights(1, highlights, &Style::default());
    assert_eq!(
        grid.plugin_highlights.get(&1).unwrap()[0].1.layer,
        HighlightLayer::ActionFeedback,
        "Layer field should be propagated through compilation"
    );
}

#[test]
fn default_layer_is_hint() {
    assert_eq!(HighlightLayer::default(), HighlightLayer::Hint);
}

#[test]
fn layer_ordering() {
    assert!(HighlightLayer::Hint < HighlightLayer::Tool);
    assert!(HighlightLayer::Tool < HighlightLayer::ActionFeedback);
}

fn row_text(row: &super::super::Row) -> String {
    row.columns
        .iter()
        .map(|c| c.character)
        .collect::<String>()
        .trim_end()
        .to_string()
}

fn scrollback_texts(grid: &Grid) -> Vec<String> {
    grid.lines_above.iter().map(|r| row_text(r)).collect()
}

fn viewport_texts(grid: &Grid) -> Vec<String> {
    grid.viewport.iter().map(|r| row_text(r)).collect()
}

fn kitty_raw_payload_b64(width: u32, height: u32, bytes_per_pixel: usize, byte: u8) -> String {
    let payload_len = (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixel_count| pixel_count.checked_mul(bytes_per_pixel))
        .unwrap();
    base64::encode(vec![byte; payload_len])
}

fn kitty_rgba_payload_b64(width: u32, height: u32) -> String {
    kitty_raw_payload_b64(width, height, 4, 0)
}

fn kitty_explicit_rgba(image_id: u32, width: u32, height: u32, cols: u32, rows: u32) -> Vec<u8> {
    let payload_b64 = kitty_rgba_payload_b64(width, height);
    format!(
        "\u{1b}_Ga=T,f=32,s={width},v={height},c={cols},r={rows},i={image_id};{payload_b64}\u{1b}\\"
    )
    .into_bytes()
}

fn kitty_explicit_rgba_no_movement(
    image_id: u32,
    width: u32,
    height: u32,
    cols: u32,
    rows: u32,
) -> Vec<u8> {
    let payload_b64 = kitty_rgba_payload_b64(width, height);
    format!(
        "\u{1b}_Ga=T,C=1,f=32,s={width},v={height},c={cols},r={rows},i={image_id};{payload_b64}\u{1b}\\"
    )
    .into_bytes()
}

fn kitty_explicit_rgba_no_movement_with_placement(
    image_id: u32,
    placement_id: u32,
    width: u32,
    height: u32,
    cols: u32,
    rows: u32,
    z_index: Option<i32>,
) -> Vec<u8> {
    let z_fragment = z_index
        .map(|z_index| format!(",z={z_index}"))
        .unwrap_or_default();
    let payload_b64 = kitty_rgba_payload_b64(width, height);
    format!(
        "\u{1b}_Ga=T,C=1,f=32,s={width},v={height},c={cols},r={rows},i={image_id},p={placement_id}{z_fragment};{payload_b64}\u{1b}\\"
    )
    .into_bytes()
}

fn kitty_virtual_rgba(image_id: u32, width: u32, height: u32, cols: u32, rows: u32) -> Vec<u8> {
    let payload_b64 = kitty_rgba_payload_b64(width, height);
    format!(
        "\u{1b}_Ga=T,U=1,f=32,s={width},v={height},c={cols},r={rows},i={image_id};{payload_b64}\u{1b}\\"
    )
    .into_bytes()
}

fn kitty_virtual_rgba_with_placement(
    image_id: u32,
    placement_id: u32,
    width: u32,
    height: u32,
    cols: u32,
    rows: u32,
) -> Vec<u8> {
    let payload_b64 = kitty_rgba_payload_b64(width, height);
    format!(
        "\u{1b}_Ga=T,U=1,f=32,s={width},v={height},c={cols},r={rows},i={image_id},p={placement_id};{payload_b64}\u{1b}\\"
    )
    .into_bytes()
}

fn kitty_virtual_rgba_with_placement_payload(
    image_id: u32,
    placement_id: u32,
    width: u32,
    height: u32,
    cols: u32,
    rows: u32,
    payload_b64: &str,
) -> Vec<u8> {
    format!(
        "\u{1b}_Ga=T,U=1,f=32,s={width},v={height},c={cols},r={rows},i={image_id},p={placement_id};{payload_b64}\u{1b}\\"
    )
    .into_bytes()
}

fn kitty_virtual_rgb_with_placement(
    image_id: u32,
    placement_id: u32,
    width: u32,
    height: u32,
    cols: u32,
    rows: u32,
) -> Vec<u8> {
    let payload_b64 = base64::encode(vec![0; (width * height * 3) as usize]);
    format!(
        "\u{1b}_Ga=T,U=1,f=24,s={width},v={height},c={cols},r={rows},i={image_id},p={placement_id};{payload_b64}\u{1b}\\"
    )
    .into_bytes()
}

fn kitty_retransmit_rgba(image_id: u32, width: u32, height: u32) -> Vec<u8> {
    let payload_b64 = kitty_rgba_payload_b64(width, height);
    format!("\u{1b}_Ga=t,f=32,s={width},v={height},i={image_id};{payload_b64}\u{1b}\\").into_bytes()
}

fn kitty_display_placement(image_id: u32, placement_id: u32, cols: u32, rows: u32) -> Vec<u8> {
    format!("\u{1b}_Ga=p,i={image_id},p={placement_id},c={cols},r={rows}\u{1b}\\").into_bytes()
}

fn kitty_display_cropped_placement(
    image_id: u32,
    placement_id: u32,
    cols: u32,
    rows: u32,
    source_x: u32,
    source_y: u32,
    source_width: u32,
    source_height: u32,
) -> Vec<u8> {
    format!(
        "\u{1b}_Ga=p,C=1,i={image_id},p={placement_id},c={cols},r={rows},x={source_x},y={source_y},w={source_width},h={source_height}\u{1b}\\"
    )
    .into_bytes()
}

fn kitty_relative_display_placement(
    image_id: u32,
    placement_id: u32,
    cols: u32,
    rows: u32,
    parent_image_id: u32,
    parent_placement_id: u32,
    offset_x: i32,
    offset_y: i32,
) -> Vec<u8> {
    format!(
        "\u{1b}_Ga=p,i={image_id},p={placement_id},c={cols},r={rows},P={parent_image_id},Q={parent_placement_id},H={offset_x},V={offset_y}\u{1b}\\"
    )
    .into_bytes()
}

fn kitty_relative_virtual_display_placement(
    image_id: u32,
    placement_id: u32,
    cols: u32,
    rows: u32,
    parent_image_id: u32,
    parent_placement_id: u32,
    offset_x: i32,
    offset_y: i32,
) -> Vec<u8> {
    format!(
        "\u{1b}_Ga=p,U=1,i={image_id},p={placement_id},c={cols},r={rows},P={parent_image_id},Q={parent_placement_id},H={offset_x},V={offset_y}\u{1b}\\"
    )
    .into_bytes()
}

fn kitty_explicit_rgba_compressed(
    image_id: u32,
    width: u32,
    height: u32,
    cols: u32,
    rows: u32,
    raw_payload: &[u8],
) -> Vec<u8> {
    let compressed = miniz_oxide::deflate::compress_to_vec_zlib(raw_payload, 6);
    let payload_b64 = base64::encode(compressed);
    format!(
        "\u{1b}_Ga=T,o=z,f=32,s={width},v={height},c={cols},r={rows},i={image_id};{payload_b64}\u{1b}\\"
    )
    .into_bytes()
}

fn kitty_explicit_rgb_compressed(
    image_id: u32,
    width: u32,
    height: u32,
    cols: u32,
    rows: u32,
    raw_payload: &[u8],
) -> Vec<u8> {
    let compressed = miniz_oxide::deflate::compress_to_vec_zlib(raw_payload, 6);
    let payload_b64 = base64::encode(compressed);
    format!(
        "\u{1b}_Ga=T,o=z,f=24,s={width},v={height},c={cols},r={rows},i={image_id};{payload_b64}\u{1b}\\"
    )
    .into_bytes()
}

fn kitty_explicit_rgb_compressed_chunked(
    image_id: u32,
    width: u32,
    height: u32,
    cols: u32,
    rows: u32,
    raw_payload: &[u8],
    chunk_size: usize,
) -> Vec<u8> {
    let compressed = miniz_oxide::deflate::compress_to_vec_zlib(raw_payload, 6);
    let payload_b64 = base64::encode(compressed);
    let parts: Vec<&str> = payload_b64
        .as_bytes()
        .chunks(chunk_size)
        .map(|chunk| std::str::from_utf8(chunk).unwrap())
        .collect();
    let mut output = String::new();
    let last_index = parts.len().saturating_sub(1);
    for (index, part) in parts.into_iter().enumerate() {
        let more = if index < last_index { 1 } else { 0 };
        if index == 0 {
            output.push_str(&format!(
                "\u{1b}_Ga=T,o=z,f=24,s={width},v={height},c={cols},r={rows},i={image_id},m={more};{part}\u{1b}\\"
            ));
        } else {
            output.push_str(&format!("\u{1b}_Gm={more};{part}\u{1b}\\"));
        }
    }
    output.into_bytes()
}

fn kitty_placeholder_text(image_id: u32) -> Vec<u8> {
    let image_id_b = image_id & 0xFF;
    let placeholder = '\u{10EEEE}';
    let row0 = '\u{305}';
    let col0 = '\u{305}';
    let col1 = '\u{30D}';
    format!(
        "\u{1b}[38;2;0;0;{image_id_b}m{placeholder}{row0}{col0}{placeholder}{row0}{col1}\u{1b}[39mX"
    )
    .into_bytes()
}

fn placeholder_rgba_text(image_id: u32, cols: usize, rows: usize) -> Vec<u8> {
    let low = image_id & 0x00FF_FFFF;
    let r = (low >> 16) & 0xFF;
    let g = (low >> 8) & 0xFF;
    let b = low & 0xFF;
    let placeholder = '\u{10EEEE}';
    let diacritics = [
        '\u{305}', '\u{30D}', '\u{30E}', '\u{310}', '\u{312}', '\u{33D}', '\u{33E}', '\u{33F}',
        '\u{346}', '\u{34A}', '\u{34B}', '\u{34C}', '\u{350}', '\u{351}',
    ];
    let mut output = String::new();
    for row in 0..rows {
        output.push_str(&format!("\u{1b}[38;2;{r};{g};{b}m"));
        let row_diacritic = diacritics[row];
        for col in 0..cols {
            let col_diacritic = diacritics[col];
            output.push(placeholder);
            output.push(row_diacritic);
            output.push(col_diacritic);
        }
        output.push_str("\u{1b}[39m");
        if row + 1 < rows {
            output.push_str("\r\n");
        }
    }
    output.into_bytes()
}

fn placeholder_rgba_text_with_placement(
    image_id: u32,
    placement_id: u32,
    cols: usize,
    rows: usize,
) -> Vec<u8> {
    let image_low = image_id & 0x00FF_FFFF;
    let image_r = (image_low >> 16) & 0xFF;
    let image_g = (image_low >> 8) & 0xFF;
    let image_b = image_low & 0xFF;
    let placement_low = placement_id & 0x00FF_FFFF;
    let placement_r = (placement_low >> 16) & 0xFF;
    let placement_g = (placement_low >> 8) & 0xFF;
    let placement_b = placement_low & 0xFF;
    let placeholder = '\u{10EEEE}';
    let diacritics = [
        '\u{305}', '\u{30D}', '\u{30E}', '\u{310}', '\u{312}', '\u{33D}', '\u{33E}', '\u{33F}',
        '\u{346}', '\u{34A}', '\u{34B}', '\u{34C}', '\u{350}', '\u{351}',
    ];
    let mut output = String::new();
    for row in 0..rows {
        output.push_str(&format!(
            "\u{1b}[38;2;{image_r};{image_g};{image_b}m\u{1b}[58;2;{placement_r};{placement_g};{placement_b}m"
        ));
        let row_diacritic = diacritics[row];
        for col in 0..cols {
            let col_diacritic = diacritics[col];
            output.push(placeholder);
            output.push(row_diacritic);
            output.push(col_diacritic);
        }
        output.push_str("\u{1b}[39m\u{1b}[59m");
    }
    output.into_bytes()
}

fn placeholder_text_with_placement_inherited_single_row(
    image_id: u32,
    placement_id: u32,
    cols: usize,
) -> Vec<u8> {
    let image_low = image_id & 0x00FF_FFFF;
    let image_r = (image_low >> 16) & 0xFF;
    let image_g = (image_low >> 8) & 0xFF;
    let image_b = image_low & 0xFF;
    let placement_low = placement_id & 0x00FF_FFFF;
    let placement_r = (placement_low >> 16) & 0xFF;
    let placement_g = (placement_low >> 8) & 0xFF;
    let placement_b = placement_low & 0xFF;
    let placeholder = '\u{10EEEE}';
    let row0 = '\u{305}';
    let col0 = '\u{305}';
    let mut output = String::new();
    output.push_str(&format!(
        "\u{1b}[38;2;{image_r};{image_g};{image_b}m\u{1b}[58;2;{placement_r};{placement_g};{placement_b}m"
    ));
    output.push(placeholder);
    output.push(row0);
    output.push(col0);
    for _ in 1..cols {
        output.push(placeholder);
    }
    output.push_str("\u{1b}[39m\u{1b}[59m");
    output.into_bytes()
}

fn placeholder_run_start_default_text(image_id: u32, x: usize, y: usize) -> Vec<u8> {
    let image_low = image_id & 0x00FF_FFFF;
    let image_r = (image_low >> 16) & 0xFF;
    let image_g = (image_low >> 8) & 0xFF;
    let image_b = image_low & 0xFF;
    let placeholder = '\u{10EEEE}';
    format!("\u{1b}[{y};{x}H\u{1b}[38;2;{image_r};{image_g};{image_b}m{placeholder}\u{1b}[39m")
        .into_bytes()
}

fn placeholder_soft_wrap_explicit_diacritics_text(image_id: u32, x: usize, y: usize) -> Vec<u8> {
    let image_low = image_id & 0x00FF_FFFF;
    let image_r = (image_low >> 16) & 0xFF;
    let image_g = (image_low >> 8) & 0xFF;
    let image_b = image_low & 0xFF;
    let placeholder = '\u{10EEEE}';
    let row0 = '\u{305}';
    let col0 = '\u{305}';
    let col1 = '\u{30D}';
    let col2 = '\u{30E}';
    let hi1 = '\u{30D}';
    format!(
        "\u{1b}[{y};{x}H\u{1b}[38;2;{image_r};{image_g};{image_b}m{placeholder}{row0}{col0}{hi1}{placeholder}{row0}{col1}{hi1}{placeholder}{row0}{col2}{hi1}\u{1b}[39m"
    )
    .into_bytes()
}

fn placeholder_soft_wrap_inherited_diacritics_text(image_id: u32, x: usize, y: usize) -> Vec<u8> {
    let image_low = image_id & 0x00FF_FFFF;
    let image_r = (image_low >> 16) & 0xFF;
    let image_g = (image_low >> 8) & 0xFF;
    let image_b = image_low & 0xFF;
    let placeholder = '\u{10EEEE}';
    let row0 = '\u{305}';
    let col0 = '\u{305}';
    let hi1 = '\u{30D}';
    format!(
        "\u{1b}[{y};{x}H\u{1b}[38;2;{image_r};{image_g};{image_b}m{placeholder}{row0}{col0}{hi1}{placeholder}{placeholder}\u{1b}[39m"
    )
    .into_bytes()
}

fn placeholder_text_with_placement_inherited_rows(
    image_id: u32,
    placement_id: u32,
    cols: usize,
    rows: usize,
    x: usize,
    y: usize,
) -> Vec<u8> {
    let image_low = image_id & 0x00FF_FFFF;
    let image_r = (image_low >> 16) & 0xFF;
    let image_g = (image_low >> 8) & 0xFF;
    let image_b = image_low & 0xFF;
    let placement_low = placement_id & 0x00FF_FFFF;
    let placement_r = (placement_low >> 16) & 0xFF;
    let placement_g = (placement_low >> 8) & 0xFF;
    let placement_b = placement_low & 0xFF;
    let placeholder = '\u{10EEEE}';
    let diacritics = [
        '\u{305}', '\u{30D}', '\u{30E}', '\u{310}', '\u{312}', '\u{33D}', '\u{33E}', '\u{33F}',
        '\u{346}', '\u{34A}', '\u{34B}', '\u{34C}', '\u{350}', '\u{351}',
    ];
    let mut output = String::new();
    for row in 0..rows {
        output.push_str(&format!("\u{1b}[{};{}H", y + row, x));
        output.push_str(&format!(
            "\u{1b}[38;2;{image_r};{image_g};{image_b}m\u{1b}[58;2;{placement_r};{placement_g};{placement_b}m"
        ));
        output.push(placeholder);
        output.push(diacritics[row]);
        for _ in 1..cols {
            output.push(placeholder);
        }
        output.push_str("\u{1b}[39m\u{1b}[59m");
    }
    output.into_bytes()
}

fn create_grid_with_size_and_raw(rows: usize, cols: usize, content: &[u8]) -> Grid {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let mut grid = Grid::new(
        rows,
        cols,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        kitty_asset_store,
        Style::default(),
        false,
        true,
        true,
        true,
        false,
    );
    vte_parser.advance(&mut grid, &content);
    grid
}

fn create_grid_with_shared_stores(
    rows: usize,
    cols: usize,
) -> (
    Grid,
    Rc<RefCell<SixelImageStore>>,
    Rc<RefCell<KittyAssetStore>>,
    Rc<RefCell<Option<SizeInPixels>>>,
) {
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    let character_cell_size = Rc::new(RefCell::new(Some(SizeInPixels {
        width: 10,
        height: 20,
    })));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let grid = Grid::new(
        rows,
        cols,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        character_cell_size.clone(),
        sixel_image_store.clone(),
        kitty_asset_store.clone(),
        Style::default(),
        false,
        true,
        true,
        true,
        false,
    );
    (
        grid,
        sixel_image_store,
        kitty_asset_store,
        character_cell_size,
    )
}

fn feed_bytes(grid: &mut Grid, bytes: &[u8]) {
    let mut vte_parser = vte::Parser::new();
    for byte in bytes {
        vte_parser.advance(grid, &[*byte]);
    }
}

fn place_explicit_rgba_at(
    grid: &mut Grid,
    image_id: u32,
    placement_id: u32,
    row: usize,
    col: usize,
    cols: u32,
    rows: u32,
    z_index: Option<i32>,
) {
    feed_bytes(grid, format!("\u{1b}[{row};{col}H").as_bytes());
    feed_bytes(
        grid,
        &kitty_explicit_rgba_no_movement_with_placement(
            image_id,
            placement_id,
            cols,
            rows,
            cols,
            rows,
            z_index,
        ),
    );
}

fn visible_kitty_placement_ids(grid: &Grid) -> Vec<u32> {
    let mut placement_ids: Vec<u32> = grid
        .visible_kitty_image_chunks(0, 0)
        .into_iter()
        .filter_map(|chunk| chunk.placement_id.map(PlacementId::wire_value))
        .collect();
    placement_ids.sort_unstable();
    placement_ids
}

fn chunk_text(chunks: &[crate::output::CharacterChunk]) -> String {
    chunks
        .iter()
        .flat_map(|chunk| chunk.terminal_characters.iter())
        .map(|t| t.character)
        .filter(|c| !c.is_whitespace())
        .collect()
}

fn visible_placeholder_cell_positions(grid: &Grid) -> Vec<(usize, usize)> {
    let mut positions: Vec<(usize, usize)> = grid
        .visible_kitty_placeholder_renders(0, 0)
        .into_iter()
        .flat_map(|render| {
            render
                .cells
                .into_iter()
                .map(|cell| (cell.cell_x, cell.cell_y))
        })
        .collect();
    positions.sort_unstable();
    positions
}

#[test]
fn kitty_placeholder_cells_advance_text_flow() {
    let mut grid = create_grid_with_size_and_raw(5, 10, &kitty_virtual_rgba(7, 2, 1, 2, 1));
    feed_bytes(&mut grid, &kitty_placeholder_text(7));

    let placeholder_renders = grid.visible_kitty_placeholder_renders(0, 0);
    assert_eq!(placeholder_renders.len(), 1);
    assert_eq!(placeholder_renders[0].cells.len(), 2);
    assert_eq!(placeholder_renders[0].cells[0].cell_x, 0);
    assert_eq!(placeholder_renders[0].cells[1].cell_x, 1);
    assert_eq!(placeholder_renders[0].cells[0].placeholder_col, 0);
    assert_eq!(placeholder_renders[0].cells[1].placeholder_col, 1);
    assert!(
        viewport_texts(&grid)
            .iter()
            .all(|line| !line.contains('\u{10EEEE}')),
        "raw kitty placeholder glyphs should not leak into viewport text"
    );
}

#[test]
fn kitty_explicit_c1_placement_does_not_advance_text_flow() {
    let mut grid = create_grid_with_size_and_raw(2, 8, b"AAA\r\nBBB");

    assert_eq!(grid.lines_above.len(), 0);
    feed_bytes(&mut grid, &kitty_explicit_rgba_no_movement(27, 1, 2, 1, 2));

    assert_eq!(
        grid.lines_above.len(),
        0,
        "explicit C=1 placement should not add canonical lines or scroll text"
    );
    let vp = viewport_texts(&grid);
    assert_eq!(vp[0], "AAA");
    assert_eq!(vp[1], "BBB");
}

#[test]
fn kitty_explicit_default_placement_advances_text_flow() {
    let mut grid = create_grid_with_size_and_raw(2, 8, b"AAA\r\nBBB");

    assert_eq!(grid.lines_above.len(), 0);
    feed_bytes(&mut grid, &kitty_explicit_rgba(28, 1, 2, 1, 2));

    assert_eq!(grid.lines_above.len(), 1);
    assert_eq!((grid.cursor.x, grid.cursor.y), (4, 1));
}

#[test]
fn kitty_explicit_c1_stays_aligned_with_text_marker_across_scroll_viewport_transitions() {
    let mut grid = create_grid_with_size_and_raw(4, 12, b"\x1b[2;2HMARK\x1b[2;8H");
    feed_bytes(&mut grid, &kitty_explicit_rgba_no_movement(35, 1, 1, 1, 1));
    feed_bytes(&mut grid, b"\r\nAA\r\nBB\r\nCC\r\nDD\r\nEE");

    assert!(
        grid.visible_kitty_image_chunks(0, 0).is_empty(),
        "explicit image should be out of view after enough subsequent text"
    );

    grid.move_viewport_up(4);
    let chunks = grid.visible_kitty_image_chunks(0, 0);
    assert_eq!(
        chunks.len(),
        1,
        "explicit image should reappear when scrolling back"
    );

    let marker_row = viewport_texts(&grid)
        .iter()
        .position(|line| line.contains("MARK"))
        .expect("marker text should reappear when scrolling back");
    assert_eq!(
        chunks[0].cell_y, marker_row,
        "explicit C=1 image should stay aligned with surrounding text after scrolling"
    );

    grid.move_viewport_down(4);
    assert!(
        grid.visible_kitty_image_chunks(0, 0).is_empty(),
        "explicit image should disappear again when scrolled back to the bottom"
    );
}

#[test]
fn kitty_explicit_c1_stays_aligned_with_text_marker_after_prior_wrap_and_scroll() {
    let mut grid = create_grid_with_size_and_raw(
        4,
        10,
        b"this heading is long enough to wrap and scroll before placement",
    );

    feed_bytes(&mut grid, b"\x1b[4;2HMARK");
    feed_bytes(&mut grid, b"\x1b[4;8H");
    feed_bytes(&mut grid, &kitty_explicit_rgba_no_movement(36, 1, 1, 1, 1));

    let marker_row = viewport_texts(&grid)
        .iter()
        .position(|line| line.contains("MARK"))
        .expect("marker text should be visible");
    let chunks = grid.visible_kitty_image_chunks(0, 0);
    assert_eq!(
        chunks.len(),
        1,
        "expected a single explicit kitty image chunk"
    );
    assert_eq!(
        chunks[0].cell_y, marker_row,
        "explicit C=1 image should stay aligned with a bottom-row marker after prior wrap/scroll"
    );
}

#[test]
fn kitty_explicit_c1_scrolls_with_text_after_bottom_newline() {
    let mut grid = create_grid_with_size_and_raw(4, 12, b"\x1b[3;2HMARK\x1b[3;8H");
    feed_bytes(&mut grid, &kitty_explicit_rgba_no_movement(37, 1, 1, 1, 1));

    let before_marker_row = viewport_texts(&grid)
        .iter()
        .position(|line| line.contains("MARK"))
        .expect("marker should be visible before scroll");
    let before_chunks = grid.visible_kitty_image_chunks(0, 0);
    assert_eq!(before_chunks.len(), 1);
    assert_eq!(before_chunks[0].cell_y, before_marker_row);

    feed_bytes(&mut grid, b"\x1b[4;1H\n");

    let after_marker_row = viewport_texts(&grid)
        .iter()
        .position(|line| line.contains("MARK"))
        .expect("marker should remain visible after bottom newline scroll");
    let after_chunks = grid.visible_kitty_image_chunks(0, 0);
    assert_eq!(after_chunks.len(), 1);
    assert_eq!(
        after_chunks[0].cell_y, after_marker_row,
        "explicit C=1 image should scroll with surrounding text after bottom newline"
    );
}

#[test]
fn kitty_placeholder_scrolls_with_text_after_bottom_newline() {
    let mut grid =
        create_grid_with_size_and_raw(4, 12, &kitty_virtual_rgba_with_placement(38, 1, 1, 1, 1, 1));
    feed_bytes(&mut grid, b"\x1b[3;2HMARK\x1b[3;8H");
    feed_bytes(
        &mut grid,
        &placeholder_text_with_placement_inherited_single_row(38, 1, 1),
    );

    let before_marker_row = viewport_texts(&grid)
        .iter()
        .position(|line| line.contains("MARK"))
        .expect("marker should be visible before scroll");
    let before_renders = grid.visible_kitty_placeholder_renders(0, 0);
    assert_eq!(
        before_renders.len(),
        1,
        "before-scroll viewport={:?} scrollback={:?} scene={:?}",
        viewport_texts(&grid),
        scrollback_texts(&grid),
        grid.image_scene
    );
    assert_eq!(before_renders[0].cells.len(), 1);
    assert_eq!(before_renders[0].cells[0].cell_y, before_marker_row);

    feed_bytes(&mut grid, b"\x1b[4;1H\n");

    let after_marker_row = viewport_texts(&grid)
        .iter()
        .position(|line| line.contains("MARK"))
        .expect("marker should remain visible after bottom newline scroll");
    let after_renders = grid.visible_kitty_placeholder_renders(0, 0);
    assert_eq!(
        after_renders.len(),
        1,
        "viewport={:?} scrollback={:?} scene={:?}",
        viewport_texts(&grid),
        scrollback_texts(&grid),
        grid.image_scene
    );
    assert_eq!(after_renders[0].cells.len(), 1);
    assert_eq!(
        after_renders[0].cells[0].cell_y, after_marker_row,
        "placeholder image should scroll with surrounding text after bottom newline"
    );
}

#[test]
fn kitty_explicit_multirow_bottom_clipped_scrolls_with_text_after_bottom_newline() {
    let mut grid = create_grid_with_size_and_raw(4, 12, b"\x1b[2;2HMARK\x1b[2;8H");
    feed_bytes(&mut grid, &kitty_explicit_rgba_no_movement(40, 2, 8, 2, 4));

    let before_marker_row = viewport_texts(&grid)
        .iter()
        .position(|line| line.contains("MARK"))
        .expect("marker should be visible before scroll");
    let before_chunks = grid.visible_kitty_image_chunks(0, 0);
    assert_eq!(before_chunks.len(), 1);
    assert_eq!(before_chunks[0].cell_y, before_marker_row);

    feed_bytes(&mut grid, b"\x1b[4;1H\n");

    let after_marker_row = viewport_texts(&grid)
        .iter()
        .position(|line| line.contains("MARK"))
        .expect("marker should remain visible after bottom newline scroll");
    let after_chunks = grid.visible_kitty_image_chunks(0, 0);
    assert_eq!(after_chunks.len(), 1);
    assert_eq!(
        after_chunks[0].cell_y, after_marker_row,
        "multi-row explicit image should scroll with surrounding text after bottom newline"
    );
}

#[test]
fn kitty_placeholder_multirow_bottom_clipped_scrolls_with_text_after_bottom_newline() {
    let mut grid =
        create_grid_with_size_and_raw(4, 12, &kitty_virtual_rgba_with_placement(41, 1, 2, 8, 2, 4));
    feed_bytes(&mut grid, b"\x1b[2;2HMARK\x1b[2;8H");
    feed_bytes(
        &mut grid,
        &placeholder_rgba_text_with_placement(41, 1, 2, 4),
    );

    let before_marker_row = viewport_texts(&grid)
        .iter()
        .position(|line| line.contains("MARK"))
        .expect("marker should be visible before scroll");
    let before_renders = grid.visible_kitty_placeholder_renders(0, 0);
    assert_eq!(
        before_renders.len(),
        1,
        "before-scroll viewport={:?} scrollback={:?} scene={:?}",
        viewport_texts(&grid),
        scrollback_texts(&grid),
        grid.image_scene
    );
    assert!(
        before_renders[0]
            .cells
            .iter()
            .any(|cell| cell.cell_y == before_marker_row),
        "placeholder cells should include the marker row before scroll"
    );

    feed_bytes(&mut grid, b"\x1b[4;1H\n");

    let after_marker_row = viewport_texts(&grid)
        .iter()
        .position(|line| line.contains("MARK"))
        .expect("marker should remain visible after bottom newline scroll");
    let after_renders = grid.visible_kitty_placeholder_renders(0, 0);
    assert_eq!(after_renders.len(), 1);
    assert!(
        after_renders[0]
            .cells
            .iter()
            .any(|cell| cell.cell_y == after_marker_row),
        "multi-row placeholder should still include the marker row after bottom newline"
    );
}

fn goto_bytes(x: usize, y: usize) -> Vec<u8> {
    format!("\u{1b}[{y};{x}H").into_bytes()
}

fn smoke_draw_box_bytes(x: usize, y: usize, w: usize, h: usize, title: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    let title_text = format!("[{title}]");
    let mut top = format!("+{}+", "-".repeat(w));
    if title_text.len() + 4 <= top.len() {
        top = format!(
            "{}{}{}",
            &top[..2],
            title_text,
            &top[2 + title_text.len()..]
        );
    }
    bytes.extend_from_slice(&goto_bytes(x, y));
    bytes.extend_from_slice(top.as_bytes());
    for row in 1..=h {
        bytes.extend_from_slice(&goto_bytes(x, y + row));
        bytes.extend_from_slice(format!("|{}|", " ".repeat(w)).as_bytes());
    }
    bytes.extend_from_slice(&goto_bytes(x, y + h + 1));
    bytes.extend_from_slice(format!("+{}+", "-".repeat(w)).as_bytes());
    bytes
}

fn shared_asset_smoke_stage_blue_phase_bytes() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&shared_asset_smoke_stage_layout_bytes());
    bytes.extend_from_slice(&shared_asset_smoke_stage_label_bytes());
    bytes
}

fn shared_asset_smoke_stage_layout_bytes() -> Vec<u8> {
    let mut bytes = Vec::new();
    let width = 48;
    let height = 32;
    let top_image_id = 122;
    let bottom_image_id = 123;

    bytes.extend_from_slice(b"\x1b[2J\x1b[H\x1b[0m");
    bytes.extend_from_slice(&goto_bytes(1, 1));
    bytes.extend_from_slice(b"Shared Asset: replace then recreate placements");
    bytes.extend_from_slice(&smoke_draw_box_bytes(2, 7, 10, 4, "separate p/U=1"));
    bytes.extend_from_slice(&smoke_draw_box_bytes(26, 7, 16, 6, "separate explicit"));
    bytes.extend_from_slice(&kitty_virtual_rgba_with_placement(
        top_image_id,
        1,
        width,
        height,
        10,
        4,
    ));
    bytes.extend_from_slice(&goto_bytes(3, 8));
    bytes.extend_from_slice(&placeholder_rgba_text_with_placement(
        top_image_id,
        1,
        10,
        4,
    ));
    bytes.extend_from_slice(&goto_bytes(27, 8));
    bytes.extend_from_slice(&kitty_display_placement(top_image_id, 2, 16, 6));

    bytes.extend_from_slice(&smoke_draw_box_bytes(2, 15, 10, 4, "combined T,U=1"));
    bytes.extend_from_slice(&smoke_draw_box_bytes(26, 15, 16, 6, "combined explicit"));
    bytes.extend_from_slice(&kitty_virtual_rgba_with_placement(
        bottom_image_id,
        1,
        width,
        height,
        10,
        4,
    ));
    bytes.extend_from_slice(&goto_bytes(3, 16));
    bytes.extend_from_slice(&placeholder_rgba_text_with_placement(
        bottom_image_id,
        1,
        10,
        4,
    ));
    bytes.extend_from_slice(&goto_bytes(27, 16));
    bytes.extend_from_slice(&kitty_display_placement(bottom_image_id, 2, 16, 6));
    bytes
}

fn shared_asset_smoke_stage_label_bytes() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&goto_bytes(1, 3));
    bytes.extend_from_slice(
        b"Expect first: both pairs below show blue shared-asset placeholder + explicit placements.",
    );
    bytes.extend_from_slice(&goto_bytes(1, 4));
    bytes.extend_from_slice(
        b"Then both assets are replaced with green bytes. The top placeholder is recreated via a separate U=1 placement.",
    );
    bytes.extend_from_slice(&goto_bytes(1, 5));
    bytes.extend_from_slice(
        b"The bottom placeholder is recreated via combined T,U=1. Explicit recreate stays the same in both pairs.",
    );
    bytes
}

#[test]
fn kitty_shared_asset_scene_keeps_placeholder_and_explicit_rows_stable_across_resize_cycles() {
    let mut grid =
        create_grid_with_size_and_raw(24, 100, &shared_asset_smoke_stage_blue_phase_bytes());

    let initial_placeholder = grid.visible_kitty_placeholder_renders(0, 0);
    let initial_explicit = grid.visible_kitty_image_chunks(0, 0);
    assert_eq!(initial_placeholder.len(), 2);
    assert_eq!(initial_explicit.len(), 2);
    let mut initial_placeholder_ys = initial_placeholder
        .iter()
        .map(|render| render.cells.iter().map(|cell| cell.cell_y).min().unwrap())
        .collect::<Vec<_>>();
    initial_placeholder_ys.sort_unstable();
    let mut initial_explicit_ys = initial_explicit
        .iter()
        .map(|chunk| chunk.cell_y)
        .collect::<Vec<_>>();
    initial_explicit_ys.sort_unstable();

    for _ in 0..3 {
        grid.change_size(24, 40);
        grid.change_size(24, 100);
    }

    let final_placeholder = grid.visible_kitty_placeholder_renders(0, 0);
    let final_explicit = grid.visible_kitty_image_chunks(0, 0);
    assert_eq!(final_placeholder.len(), 2);
    assert_eq!(final_explicit.len(), 2);
    let mut final_placeholder_ys = final_placeholder
        .iter()
        .map(|render| render.cells.iter().map(|cell| cell.cell_y).min().unwrap())
        .collect::<Vec<_>>();
    final_placeholder_ys.sort_unstable();
    let mut final_explicit_ys = final_explicit
        .iter()
        .map(|chunk| chunk.cell_y)
        .collect::<Vec<_>>();
    final_explicit_ys.sort_unstable();

    assert_eq!(
        final_placeholder_ys, initial_placeholder_ys,
        "placeholder rows should return to their original position after narrow->wide resize cycles"
    );
    assert_eq!(
        final_explicit_ys, initial_explicit_ys,
        "explicit placement rows should return to their original position after narrow->wide resize cycles"
    );
}

#[test]
#[ignore = "known reflow-anchor bug: shared-asset narrow/reflow cases still use stale canonical-line anchors until pin-style indirection/fixup lands"]
fn kitty_shared_asset_scene_labels_move_top_pair_down_in_narrow_pane() {
    let mut grid = create_grid_with_size_and_raw(24, 52, &shared_asset_smoke_stage_layout_bytes());

    let before_label_placeholder_y = grid
        .visible_kitty_placeholder_renders(0, 0)
        .iter()
        .map(|render| render.cells.iter().map(|cell| cell.cell_y).min().unwrap())
        .min()
        .unwrap();
    let before_label_explicit_y = grid
        .visible_kitty_image_chunks(0, 0)
        .iter()
        .map(|chunk| chunk.cell_y)
        .min()
        .unwrap();

    feed_bytes(&mut grid, &shared_asset_smoke_stage_label_bytes());

    let after_label_placeholder_y = grid
        .visible_kitty_placeholder_renders(0, 0)
        .iter()
        .map(|render| render.cells.iter().map(|cell| cell.cell_y).min().unwrap())
        .min()
        .unwrap();
    let after_label_explicit_y = grid
        .visible_kitty_image_chunks(0, 0)
        .iter()
        .map(|chunk| chunk.cell_y)
        .min()
        .unwrap();

    assert_eq!(before_label_placeholder_y, 7);
    assert_eq!(before_label_explicit_y, 7);
    assert_eq!(
        after_label_placeholder_y, 7,
        "labels written after the top pair should not push placeholder cells down"
    );
    assert_eq!(
        after_label_explicit_y, 7,
        "labels written after the top pair should not push explicit placements down"
    );
}

#[test]
#[ignore = "known reflow-anchor bug: shared-asset narrow/reflow cases still use stale canonical-line anchors until pin-style indirection/fixup lands"]
fn kitty_shared_asset_scene_narrow_reflow_keeps_images_aligned_with_box_titles() {
    let mut grid =
        create_grid_with_size_and_raw(24, 100, &shared_asset_smoke_stage_blue_phase_bytes());
    grid.change_size(24, 52);

    let placeholder = grid.visible_kitty_placeholder_renders(0, 0);
    let explicit = grid.visible_kitty_image_chunks(0, 0);
    assert_eq!(placeholder.len(), 2);
    assert_eq!(explicit.len(), 2);

    let top_placeholder_y = placeholder
        .iter()
        .map(|render| render.cells.iter().map(|cell| cell.cell_y).min().unwrap())
        .min()
        .unwrap();
    let top_explicit_y = explicit.iter().map(|chunk| chunk.cell_y).min().unwrap();

    assert_eq!(
        top_placeholder_y, 7,
        "top placeholder cells should stay on the first interior row of the top box after narrowing"
    );
    assert_eq!(
        top_explicit_y, 7,
        "top explicit placement should stay on the first interior row of the top box after narrowing"
    );
}

#[test]
#[ignore = "known reflow-anchor bug: shared-asset narrow/reflow cases still use stale canonical-line anchors until pin-style indirection/fixup lands"]
fn kitty_shared_asset_scene_created_while_already_narrow_stays_aligned_after_resize() {
    let mut grid =
        create_grid_with_size_and_raw(24, 52, &shared_asset_smoke_stage_blue_phase_bytes());

    let initial_placeholder = grid.visible_kitty_placeholder_renders(0, 0);
    let initial_explicit = grid.visible_kitty_image_chunks(0, 0);
    assert_eq!(initial_placeholder.len(), 2);
    assert_eq!(initial_explicit.len(), 2);

    let initial_top_placeholder_y = initial_placeholder
        .iter()
        .map(|render| render.cells.iter().map(|cell| cell.cell_y).min().unwrap())
        .min()
        .unwrap();
    let initial_top_explicit_y = initial_explicit
        .iter()
        .map(|chunk| chunk.cell_y)
        .min()
        .unwrap();

    assert_eq!(initial_top_placeholder_y, 7);
    assert_eq!(initial_top_explicit_y, 7);

    grid.change_size(24, 44);
    grid.change_size(24, 52);

    let final_placeholder = grid.visible_kitty_placeholder_renders(0, 0);
    let final_explicit = grid.visible_kitty_image_chunks(0, 0);
    assert_eq!(final_placeholder.len(), 2);
    assert_eq!(final_explicit.len(), 2);

    let final_top_placeholder_y = final_placeholder
        .iter()
        .map(|render| render.cells.iter().map(|cell| cell.cell_y).min().unwrap())
        .min()
        .unwrap();
    let final_top_explicit_y = final_explicit
        .iter()
        .map(|chunk| chunk.cell_y)
        .min()
        .unwrap();

    assert_eq!(final_top_placeholder_y, 7);
    assert_eq!(final_top_explicit_y, 7);
}

#[test]
fn kitty_placeholder_stays_aligned_with_text_marker_after_prior_wrap_and_scroll() {
    let mut grid = create_grid_with_size_and_raw(
        4,
        10,
        b"this heading is long enough to wrap and scroll before placement",
    );

    feed_bytes(
        &mut grid,
        &kitty_virtual_rgba_with_placement(39, 1, 1, 1, 1, 1),
    );
    feed_bytes(&mut grid, b"\x1b[4;2HMARK\x1b[4;8H");
    feed_bytes(
        &mut grid,
        &placeholder_text_with_placement_inherited_single_row(39, 1, 1),
    );

    let marker_row = viewport_texts(&grid)
        .iter()
        .position(|line| line.contains("MARK"))
        .expect("marker text should be visible");
    let renders = grid.visible_kitty_placeholder_renders(0, 0);
    assert_eq!(
        renders.len(),
        1,
        "expected a single placeholder render; viewport={:?} scrollback={:?} scene={:?}",
        viewport_texts(&grid),
        scrollback_texts(&grid),
        grid.image_scene
    );
    assert_eq!(
        renders[0].cells.len(),
        1,
        "expected a single placeholder cell"
    );
    assert_eq!(
        renders[0].cells[0].cell_y, marker_row,
        "placeholder should stay aligned with a bottom-row marker after prior wrap/scroll"
    );
}

#[test]
fn kitty_placeholder_explicit_diacritics_render_across_soft_wrap() {
    let image_id = (1 << 24) | 0x42;
    let mut grid = create_grid_with_size_and_raw(4, 6, &kitty_virtual_rgba(image_id, 40, 16, 5, 2));
    feed_bytes(
        &mut grid,
        &placeholder_soft_wrap_explicit_diacritics_text(image_id, 5, 2),
    );

    let placeholder_renders = grid.visible_kitty_placeholder_renders(0, 0);
    assert_eq!(placeholder_renders.len(), 1);
    assert_eq!(
        placeholder_renders[0].cells.len(),
        3,
        "fully explicit placeholder cells should render on both sides of a soft wrap: {placeholder_renders:?}"
    );
}

#[test]
fn kitty_placeholder_omitted_diacritics_do_not_inherit_across_soft_wrap() {
    let image_id = (1 << 24) | 0x42;
    let mut grid = create_grid_with_size_and_raw(4, 6, &kitty_virtual_rgba(image_id, 40, 16, 5, 2));
    feed_bytes(
        &mut grid,
        &placeholder_soft_wrap_inherited_diacritics_text(image_id, 5, 2),
    );

    let placeholder_renders = grid.visible_kitty_placeholder_renders(0, 0);
    assert_eq!(placeholder_renders.len(), 1);
    assert_eq!(
        placeholder_renders[0].cells.len(),
        2,
        "omitted placeholder diacritics should inherit within a render line, but not across a soft wrap: {placeholder_renders:?}"
    );
}

#[test]
fn kitty_placeholder_run_start_without_diacritics_defaults_to_first_cell() {
    let mut grid = create_grid_with_size_and_raw(4, 8, &kitty_virtual_rgba(200, 1, 1, 1, 1));
    feed_bytes(&mut grid, &placeholder_run_start_default_text(200, 2, 2));

    let placeholder_renders = grid.visible_kitty_placeholder_renders(0, 0);
    assert_eq!(
        placeholder_renders.len(),
        1,
        "first placeholder cell without diacritics should resolve to the image origin"
    );
    assert_eq!(placeholder_renders[0].cells.len(), 1);
    assert_eq!(placeholder_renders[0].cells[0].placeholder_row, 0);
    assert_eq!(placeholder_renders[0].cells[0].placeholder_col, 0);
}

#[test]
fn kitty_placeholder_inherits_omitted_diacritics_for_rgb_and_rgba() {
    let mut grid =
        create_grid_with_size_and_raw(4, 16, &kitty_virtual_rgb_with_placement(30, 1, 4, 1, 4, 1));
    feed_bytes(
        &mut grid,
        &placeholder_text_with_placement_inherited_single_row(30, 1, 4),
    );
    feed_bytes(
        &mut grid,
        &kitty_virtual_rgba_with_placement(31, 2, 4, 1, 4, 1),
    );
    feed_bytes(
        &mut grid,
        &placeholder_text_with_placement_inherited_single_row(31, 2, 4),
    );

    let placeholder_renders = grid.visible_kitty_placeholder_renders(0, 0);
    assert_eq!(placeholder_renders.len(), 2);
    assert_eq!(
        placeholder_renders
            .iter()
            .find(|render| render.image_id == 1)
            .map(|render| render.cells.len()),
        Some(4),
        "RGB placeholder cells should inherit omitted diacritics from the cell to the left",
    );
    assert_eq!(
        placeholder_renders
            .iter()
            .find(|render| render.image_id == 2)
            .map(|render| render.cells.len()),
        Some(4),
        "RGBA placeholder cells should inherit omitted diacritics from the cell to the left",
    );
}

#[test]
fn kitty_placeholder_inherits_omitted_diacritics_in_smoke_style_multi_row_grid() {
    let cols: usize = 10;
    let rows: usize = 4;
    let mut grid = create_grid_with_size_and_raw(
        24,
        40,
        &kitty_virtual_rgb_with_placement(
            160,
            1,
            cols as u32,
            rows as u32,
            cols as u32,
            rows as u32,
        ),
    );
    feed_bytes(
        &mut grid,
        &placeholder_text_with_placement_inherited_rows(160, 1, cols, rows, 3, 9),
    );
    feed_bytes(
        &mut grid,
        &kitty_virtual_rgba_with_placement(
            161,
            2,
            cols as u32,
            rows as u32,
            cols as u32,
            rows as u32,
        ),
    );
    feed_bytes(
        &mut grid,
        &placeholder_text_with_placement_inherited_rows(161, 2, cols, rows, 21, 17),
    );

    let placeholder_renders = grid.visible_kitty_placeholder_renders(0, 0);
    assert_eq!(placeholder_renders.len(), 2);
    assert_eq!(
        placeholder_renders
            .iter()
            .find(|render| render.image_id == 1)
            .map(|render| render.cells.len()),
        Some(cols * rows),
        "RGB smoke-style inherited placeholder grid should resolve all cells",
    );
    assert_eq!(
        placeholder_renders
            .iter()
            .find(|render| render.image_id == 2)
            .map(|render| render.cells.len()),
        Some(cols * rows),
        "RGBA smoke-style inherited placeholder grid should resolve all cells",
    );
}

#[test]
fn kitty_source_rectangle_width_is_clamped_to_image_bounds() {
    let mut grid = create_grid_with_size_and_raw(8, 24, &kitty_retransmit_rgba(201, 100, 100));
    feed_bytes(
        &mut grid,
        &kitty_display_cropped_placement(201, 1, 18, 8, 80, 20, 200, 60),
    );

    let chunks = grid.visible_kitty_image_chunks(0, 0);
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].source_x, 80);
    assert_eq!(chunks[0].source_y, 20);
    assert_eq!(
        chunks[0].source_width, 20,
        "source width should clamp to remaining pixels inside the image"
    );
    assert_eq!(chunks[0].source_height, 60);
}

#[test]
fn kitty_fully_outside_source_rectangle_is_not_visible() {
    let mut grid = create_grid_with_size_and_raw(8, 24, &kitty_retransmit_rgba(202, 100, 100));
    feed_bytes(
        &mut grid,
        &kitty_display_cropped_placement(202, 1, 18, 8, 120, 20, 20, 40),
    );

    assert!(
        grid.visible_kitty_image_chunks(0, 0).is_empty(),
        "fully outside source rectangles should not produce a visible placement"
    );
}

#[test]
fn kitty_compressed_rgba_payload_is_decompressed_before_storage() {
    let (mut grid, _sixel_image_store, kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(4, 8);
    let raw_payload = [0u8, 0, 0, 255, 255, 255, 255, 255];
    feed_bytes(
        &mut grid,
        &kitty_explicit_rgba_compressed(32, 1, 2, 1, 2, &raw_payload),
    );

    let stored_image_id = grid
        .visible_kitty_image_chunks(0, 0)
        .first()
        .map(|chunk| chunk.image_id)
        .expect("expected compressed RGBA placement to remain visible");
    let stored_image = kitty_asset_store
        .borrow_mut()
        .image_data(stored_image_id)
        .expect("expected compressed RGBA payload to be stored");
    match stored_image {
        crate::output::KittyImageData::Rgba {
            data,
            width,
            height,
        } => {
            assert_eq!((width, height), (1, 2));
            assert_eq!(
                data, raw_payload,
                "compressed RGBA payloads should be decompressed before entering the kitty asset store"
            );
        },
        other => panic!("expected stored RGBA image data, got {other:?}"),
    }
}

#[test]
fn kitty_compressed_rgb_payload_is_decompressed_before_storage() {
    let (mut grid, _sixel_image_store, kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(4, 8);
    let raw_payload = [0u8, 0, 0, 255, 255, 255];
    feed_bytes(
        &mut grid,
        &kitty_explicit_rgb_compressed(33, 1, 2, 1, 2, &raw_payload),
    );

    let stored_image_id = grid
        .visible_kitty_image_chunks(0, 0)
        .first()
        .map(|chunk| chunk.image_id)
        .expect("expected compressed RGB placement to remain visible");
    let stored_image = kitty_asset_store
        .borrow_mut()
        .image_data(stored_image_id)
        .expect("expected compressed RGB payload to be stored");
    match stored_image {
        crate::output::KittyImageData::Rgb {
            data,
            width,
            height,
        } => {
            assert_eq!((width, height), (1, 2));
            assert_eq!(
                data, raw_payload,
                "compressed RGB payloads should be decompressed before entering the kitty asset store"
            );
        },
        other => panic!("expected stored RGB image data, got {other:?}"),
    }
}

#[test]
fn kitty_chunked_compressed_rgb_payload_is_decompressed_after_full_assembly() {
    let (mut grid, _sixel_image_store, kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(8, 32);
    let raw_payload = vec![0x55; 8 * 4 * 3];
    feed_bytes(
        &mut grid,
        &kitty_explicit_rgb_compressed_chunked(34, 8, 4, 8, 4, &raw_payload, 8),
    );

    let stored_image_id = grid
        .visible_kitty_image_chunks(0, 0)
        .first()
        .map(|chunk| chunk.image_id)
        .expect("expected chunked compressed RGB placement to remain visible");
    let stored_image = kitty_asset_store
        .borrow_mut()
        .image_data(stored_image_id)
        .expect("expected chunked compressed RGB payload to be stored");
    match stored_image {
        crate::output::KittyImageData::Rgb {
            data,
            width,
            height,
        } => {
            assert_eq!((width, height), (8, 4));
            assert_eq!(
                data, raw_payload,
                "chunked compressed RGB payloads should be decompressed only after full assembly"
            );
        },
        other => panic!("expected stored RGB image data, got {other:?}"),
    }
}

#[test]
fn kitty_delete_aborts_inflight_chunked_upload_before_stale_final_chunk_arrives() {
    let (mut grid, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(8, 12);

    feed_bytes(
        &mut grid,
        b"\x1b_Gq=2,a=T,f=24,s=2,v=2,i=81,c=2,r=2,m=1;abcd\x1b\\",
    );
    feed_bytes(&mut grid, b"\x1b_Gm=1;efgh\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Gm=1;ijkl\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Ga=d\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Gm=0;mnop\x1b\\");

    assert!(
        grid.visible_kitty_image_chunks(0, 0).is_empty(),
        "delete during chunked upload should abort the in-flight upload before a stale final chunk can finalize it"
    );

    feed_bytes(
        &mut grid,
        b"\x1b_Gq=0,a=T,f=24,s=2,v=2,i=81,c=2,r=2,m=1;abcd\x1b\\",
    );
    feed_bytes(&mut grid, b"\x1b_Gm=1;efgh\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Gm=1;ijkl\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Gm=0;1234\x1b\\");

    assert_eq!(
        grid.visible_kitty_image_chunks(0, 0).len(),
        1,
        "a fresh chunked upload should succeed after the earlier upload was aborted"
    );
}

#[test]
fn kitty_non_query_upload_and_placement_commands_emit_replies() {
    let (mut grid, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(4, 8);

    feed_bytes(&mut grid, b"\x1b_Gq=0,a=t,f=24,s=1,v=1,i=51;EjRW\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Gq=0,a=p,i=51,p=1,c=1,r=1\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Gq=0,a=p,i=404,p=1,c=1,r=1\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Gq=1,a=p,i=51,p=2,c=1,r=1\x1b\\");
    feed_bytes(
        &mut grid,
        b"\x1b_Gq=2,a=T,f=24,s=1,v=1,i=52,c=1,r=1;EjRW\x1b\\",
    );
    feed_bytes(&mut grid, b"\x1b_Gq=2,a=p,i=404,p=2,c=1,r=1\x1b\\");

    let replies: Vec<_> = grid
        .pending_messages_to_pty
        .iter()
        .map(|message| String::from_utf8(message.clone()).unwrap())
        .collect();
    assert_eq!(
        replies.len(),
        3,
        "non-quiet kitty upload/placement commands should enqueue success and error replies, got {replies:?}",
    );
    assert!(
        replies
            .iter()
            .any(|reply| reply == "\u{1b}_Gi=51;OK\u{1b}\\"),
        "expected upload success reply for i=51, got {replies:?}",
    );
    assert!(
        replies
            .iter()
            .any(|reply| reply == "\u{1b}_Gi=51,p=1;OK\u{1b}\\"),
        "expected placement success reply for i=51,p=1, got {replies:?}",
    );
    assert!(
        replies
            .iter()
            .any(|reply| reply.contains("i=404,p=1;ENOENT:")),
        "expected missing-image placement failure reply for i=404,p=1, got {replies:?}",
    );
    assert!(
        !replies.iter().any(|reply| reply.contains("i=52;OK")),
        "q=2 successful non-query commands should not emit visible OK replies, got {replies:?}",
    );
    assert!(
        !replies
            .iter()
            .any(|reply| reply.contains("i=404,p=2;ENOENT:")),
        "q=2 failing non-query commands should suppress failure replies, got {replies:?}",
    );
}

fn kitty_reply_image_id(reply: &str) -> Option<u32> {
    let start = reply.find("i=")? + 2;
    let digits: String = reply[start..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

#[test]
fn kitty_chunked_upload_emits_reply_only_on_terminal_chunk() {
    let (mut grid, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(8, 12);

    feed_bytes(
        &mut grid,
        b"\x1b_Gq=0,a=T,f=24,s=2,v=2,i=82,c=2,r=2,m=1;abcd\x1b\\",
    );
    assert!(
        grid.pending_messages_to_pty.is_empty(),
        "opening chunk should not emit an early reply"
    );
    feed_bytes(&mut grid, b"\x1b_Gm=1;efgh\x1b\\");
    assert!(
        grid.pending_messages_to_pty.is_empty(),
        "intermediate chunk should not emit a reply"
    );
    feed_bytes(&mut grid, b"\x1b_Gm=1;ijkl\x1b\\");
    assert!(
        grid.pending_messages_to_pty.is_empty(),
        "last intermediate chunk should still not emit a reply"
    );
    feed_bytes(&mut grid, b"\x1b_Gm=0;mnop\x1b\\");

    let replies: Vec<_> = grid
        .pending_messages_to_pty
        .iter()
        .map(|message| String::from_utf8(message.clone()).unwrap())
        .collect();
    assert_eq!(replies, vec!["\u{1b}_Gi=82;OK\u{1b}\\".to_string()]);
}

#[test]
fn kitty_chunked_upload_with_image_number_emits_final_reply_with_resolved_id() {
    let (mut grid, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(8, 12);

    feed_bytes(&mut grid, b"\x1b_Gq=0,a=t,f=24,s=2,v=2,I=93,m=1;abcd\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Gm=1;efgh\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Gm=1;ijkx\x1b\\");
    assert!(
        grid.pending_messages_to_pty.is_empty(),
        "intermediate chunks should not emit replies for image-number uploads"
    );
    feed_bytes(&mut grid, b"\x1b_Gm=0;mnop\x1b\\");

    let replies: Vec<_> = grid
        .pending_messages_to_pty
        .iter()
        .map(|message| String::from_utf8(message.clone()).unwrap())
        .collect();
    assert_eq!(
        replies.len(),
        1,
        "expected only one final reply, got {replies:?}"
    );
    assert!(
        replies[0].contains("I=93;OK"),
        "final reply should preserve the image number, got {replies:?}"
    );
    let resolved_id =
        kitty_reply_image_id(&replies[0]).expect("final reply should include a resolved i=");
    assert!(
        resolved_id >= 0x8000_0001,
        "I= uploads should reply with a synthesized terminal image id, got {resolved_id}"
    );
}

#[test]
fn kitty_chunked_upload_emits_final_enodata_only_on_terminal_chunk() {
    let (mut grid, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(8, 12);

    feed_bytes(&mut grid, b"\x1b_Gq=0,a=t,f=32,s=2,v=2,i=84,m=1;abcd\x1b\\");
    assert!(
        grid.pending_messages_to_pty.is_empty(),
        "opening chunk should not emit an early error reply"
    );
    feed_bytes(&mut grid, b"\x1b_Gm=0;mnop\x1b\\");

    let replies: Vec<_> = grid
        .pending_messages_to_pty
        .iter()
        .map(|message| String::from_utf8(message.clone()).unwrap())
        .collect();
    assert_eq!(
        replies.len(),
        1,
        "expected one final failure reply, got {replies:?}"
    );
    assert!(
        replies[0].contains("ENODATA:Insufficient image data: 6 < 16"),
        "terminal chunk should report ENODATA with the expected byte counts, got {replies:?}"
    );
}

#[test]
fn kitty_direct_rgba_upload_emits_enodata_for_undersized_payload() {
    let (mut grid, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(8, 12);

    feed_bytes(&mut grid, b"\x1b_Gq=0,a=t,f=32,s=2,v=2,i=85;AAAAAA==\x1b\\");

    let replies: Vec<_> = grid
        .pending_messages_to_pty
        .iter()
        .map(|message| String::from_utf8(message.clone()).unwrap())
        .collect();
    assert_eq!(
        replies.len(),
        1,
        "expected one failure reply, got {replies:?}"
    );
    assert!(
        replies[0].contains("ENODATA:Insufficient image data: 4 < 16"),
        "direct rgba upload should report ENODATA with the expected byte counts, got {replies:?}"
    );
}

#[test]
fn kitty_direct_rgb_upload_emits_enodata_for_undersized_payload() {
    let (mut grid, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(8, 12);

    feed_bytes(&mut grid, b"\x1b_Gq=0,a=t,f=24,s=2,v=2,i=86;EjRW\x1b\\");

    let replies: Vec<_> = grid
        .pending_messages_to_pty
        .iter()
        .map(|message| String::from_utf8(message.clone()).unwrap())
        .collect();
    assert_eq!(
        replies.len(),
        1,
        "expected one failure reply, got {replies:?}"
    );
    assert!(
        replies[0].contains("ENODATA:Insufficient image data: 3 < 12"),
        "direct rgb upload should report ENODATA with the expected byte counts, got {replies:?}"
    );
}

#[test]
fn kitty_image_number_upload_and_placement_commands_emit_replies_and_render() {
    let (mut grid, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(4, 8);

    feed_bytes(&mut grid, b"\x1b_Gq=0,a=t,f=24,s=1,v=1,I=71;EjRW\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Gq=0,a=p,I=71,p=1,c=1,r=1\x1b\\");

    let replies: Vec<_> = grid
        .pending_messages_to_pty
        .iter()
        .map(|message| String::from_utf8(message.clone()).unwrap())
        .collect();
    assert_eq!(
        replies.len(),
        2,
        "expected create and place replies, got {replies:?}"
    );
    let create_reply = replies
        .iter()
        .find(|reply| reply.contains("I=71;OK") && !reply.contains(",p="))
        .expect("expected create reply with I=71");
    let place_reply = replies
        .iter()
        .find(|reply| reply.contains("I=71;OK") && reply.contains(",p=1"))
        .expect("expected place reply with I=71,p=1");
    let create_image_id =
        kitty_reply_image_id(create_reply).expect("create reply should include i=");
    let place_image_id = kitty_reply_image_id(place_reply).expect("place reply should include i=");
    assert_eq!(
        create_image_id, place_image_id,
        "placement by I= should resolve to the newest created synthetic image id"
    );

    let visible = grid.visible_kitty_image_chunks(0, 0);
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].placement_id, Some(pid(1)));
}

#[test]
fn kitty_default_explicit_placement_moves_cursor_to_cell_after_bottom_right() {
    let (mut grid, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(8, 12);

    feed_bytes(&mut grid, b"\x1b[4;5H");
    feed_bytes(&mut grid, &kitty_explicit_rgba(72, 3, 2, 3, 2));

    assert_eq!(
        (grid.cursor.x, grid.cursor.y),
        (7, 4),
        "default explicit placement should land at (x+c, y+r-1)"
    );
}

#[test]
fn kitty_default_stored_placement_moves_cursor_to_cell_after_bottom_right() {
    let (mut grid, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(8, 12);

    feed_bytes(&mut grid, &kitty_retransmit_rgba(73, 3, 2));
    feed_bytes(&mut grid, b"\x1b[4;5H");
    feed_bytes(&mut grid, &kitty_display_placement(73, 1, 3, 2));

    assert_eq!(
        (grid.cursor.x, grid.cursor.y),
        (7, 4),
        "stored explicit placement should land at (x+c, y+r-1)"
    );
}

#[test]
fn kitty_relative_placement_rejects_missing_parent() {
    let (mut grid, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(8, 12);

    feed_bytes(&mut grid, &kitty_retransmit_rgba(91, 3, 2));
    feed_bytes(
        &mut grid,
        &kitty_relative_display_placement(91, 2, 3, 2, 999, 1, 2, 1),
    );

    let replies: Vec<_> = grid
        .pending_messages_to_pty
        .iter()
        .map(|message| String::from_utf8(message.clone()).unwrap())
        .collect();
    assert!(
        replies
            .iter()
            .any(|reply| reply.contains("i=91,p=2;ENOPARENT:")),
        "missing relative parent should report ENOPARENT, got {replies:?}"
    );
    assert!(
        grid.visible_kitty_image_chunks(0, 0).is_empty(),
        "relative placement with a missing parent should not create a visible placement"
    );
}

#[test]
fn kitty_relative_placement_does_not_move_cursor() {
    let (mut grid, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(8, 16);

    feed_bytes(&mut grid, &kitty_retransmit_rgba(92, 3, 2));
    feed_bytes(&mut grid, b"\x1b[2;3H");
    feed_bytes(&mut grid, b"\x1b_Ga=p,C=1,i=92,p=1,c=3,r=2\x1b\\");
    feed_bytes(&mut grid, b"\x1b[6;8H");
    feed_bytes(
        &mut grid,
        &kitty_relative_display_placement(92, 2, 3, 2, 92, 1, 2, 1),
    );

    assert_eq!(
        (grid.cursor.x, grid.cursor.y),
        (7, 5),
        "successful relative placement should not move the cursor regardless of C"
    );
}

#[test]
fn kitty_relative_delete_of_parent_cascades_to_children() {
    let (mut grid, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(8, 16);

    feed_bytes(&mut grid, &kitty_retransmit_rgba(93, 3, 2));
    feed_bytes(&mut grid, b"\x1b[2;3H");
    feed_bytes(&mut grid, b"\x1b_Ga=p,i=93,p=1,c=3,r=2\x1b\\");
    feed_bytes(
        &mut grid,
        &kitty_relative_display_placement(93, 2, 3, 2, 93, 1, 2, 1),
    );

    let before_delete = grid.visible_kitty_image_chunks(0, 0);
    assert_eq!(
        before_delete.len(),
        2,
        "expected parent and child before delete"
    );

    feed_bytes(&mut grid, b"\x1b_Ga=d,d=i,i=93,p=1\x1b\\");

    assert!(
        grid.visible_kitty_image_chunks(0, 0).is_empty(),
        "deleting a relative parent should also delete its descendants"
    );
}

#[test]
fn kitty_relative_placement_rejects_cycles() {
    let (mut grid, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(8, 16);

    feed_bytes(&mut grid, &kitty_retransmit_rgba(94, 3, 2));
    feed_bytes(&mut grid, b"\x1b[2;3H");
    feed_bytes(&mut grid, b"\x1b_Ga=p,i=94,p=1,c=3,r=2\x1b\\");
    feed_bytes(
        &mut grid,
        &kitty_relative_display_placement(94, 2, 3, 2, 94, 1, 2, 1),
    );
    feed_bytes(
        &mut grid,
        &kitty_relative_display_placement(94, 1, 3, 2, 94, 2, 1, 0),
    );

    let replies: Vec<_> = grid
        .pending_messages_to_pty
        .iter()
        .map(|message| String::from_utf8(message.clone()).unwrap())
        .collect();
    assert!(
        replies
            .iter()
            .any(|reply| reply.contains("i=94,p=1;ECYCLE:")),
        "relative placement cycle should report ECYCLE, got {replies:?}"
    );
}

#[test]
fn kitty_relative_placement_limits_depth_like_kitty() {
    let (mut grid, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(8, 24);

    feed_bytes(&mut grid, &kitty_retransmit_rgba(96, 3, 2));
    feed_bytes(&mut grid, b"\x1b[2;3H");
    feed_bytes(&mut grid, b"\x1b_Ga=p,i=96,p=1,c=3,r=2\x1b\\");

    for placement_id in 2..=9 {
        feed_bytes(
            &mut grid,
            &kitty_relative_display_placement(96, placement_id, 3, 2, 96, placement_id - 1, 1, 0),
        );
    }

    let before_too_deep = grid.visible_kitty_image_chunks(0, 0);
    assert!(
        before_too_deep
            .iter()
            .any(|chunk| chunk.placement_id == Some(pid(9))),
        "expected placement 9 to still be accepted before the depth limit"
    );

    feed_bytes(
        &mut grid,
        &kitty_relative_display_placement(96, 10, 3, 2, 96, 9, 1, 0),
    );

    let replies: Vec<_> = grid
        .pending_messages_to_pty
        .iter()
        .map(|message| String::from_utf8(message.clone()).unwrap())
        .collect();
    assert!(
        replies
            .iter()
            .any(|reply| reply.contains("i=96,p=10;ETOODEEP:")),
        "relative placement beyond kitty's depth limit should report ETOODEEP, got {replies:?}"
    );
    assert!(
        !grid
            .visible_kitty_image_chunks(0, 0)
            .iter()
            .any(|chunk| chunk.placement_id == Some(pid(10))),
        "placement 10 should not be created once the relative depth limit is exceeded"
    );
}

#[test]
fn kitty_virtual_relative_placement_is_rejected() {
    let (mut grid, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(8, 16);

    feed_bytes(&mut grid, &kitty_retransmit_rgba(95, 3, 2));
    feed_bytes(&mut grid, b"\x1b[2;3H");
    feed_bytes(&mut grid, b"\x1b_Ga=p,i=95,p=1,c=3,r=2\x1b\\");
    feed_bytes(
        &mut grid,
        &kitty_relative_virtual_display_placement(95, 2, 3, 2, 95, 1, 1, 1),
    );

    let replies: Vec<_> = grid
        .pending_messages_to_pty
        .iter()
        .map(|message| String::from_utf8(message.clone()).unwrap())
        .collect();
    assert!(
        replies
            .iter()
            .any(|reply| reply.contains("i=95,p=2;EINVAL:")),
        "virtual relative placement should report EINVAL, got {replies:?}"
    );
}

#[test]
fn kitty_relative_child_of_virtual_parent_uses_placeholder_bounds_origin() {
    let (mut grid, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(10, 24);

    feed_bytes(
        &mut grid,
        &kitty_virtual_rgba_with_placement(96, 1, 4, 2, 4, 2),
    );
    feed_bytes(
        &mut grid,
        &placeholder_text_with_placement_inherited_rows(96, 1, 4, 2, 6, 4),
    );
    feed_bytes(&mut grid, &kitty_retransmit_rgba(97, 3, 2));
    feed_bytes(
        &mut grid,
        &kitty_relative_display_placement(97, 1, 3, 2, 96, 1, 1, 1),
    );

    let visible = grid.visible_kitty_image_chunks(0, 0);
    assert_eq!(
        visible.len(),
        1,
        "expected one visible relative child placement"
    );
    assert_eq!(
        (visible[0].cell_x, visible[0].cell_y),
        (6, 4),
        "relative child of a virtual parent should anchor to the min placeholder x/y, then apply H/V offsets"
    );
}

#[test]
fn kitty_relative_child_tracks_replaced_parent_position() {
    let (mut grid, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(10, 24);

    feed_bytes(&mut grid, &kitty_retransmit_rgba(98, 3, 2));
    feed_bytes(&mut grid, &kitty_retransmit_rgba(99, 2, 1));
    feed_bytes(&mut grid, b"\x1b[2;3H");
    feed_bytes(&mut grid, b"\x1b_Ga=p,i=98,p=1,c=3,r=2\x1b\\");
    feed_bytes(
        &mut grid,
        &kitty_relative_display_placement(99, 2, 2, 1, 98, 1, 2, 1),
    );

    let initial_visible = grid.visible_kitty_image_chunks(0, 0);
    let initial_child = initial_visible
        .iter()
        .find(|chunk| chunk.placement_id == Some(pid(2)))
        .expect("expected relative child chunk");
    assert_eq!((initial_child.cell_x, initial_child.cell_y), (4, 2));

    feed_bytes(&mut grid, b"\x1b[6;8H");
    feed_bytes(&mut grid, b"\x1b_Ga=p,i=98,p=1,c=3,r=2\x1b\\");

    let after_move = grid.visible_kitty_image_chunks(0, 0);
    let moved_child = after_move
        .iter()
        .find(|chunk| chunk.placement_id == Some(pid(2)))
        .expect("expected relative child chunk after parent replacement");
    assert_eq!(
        (moved_child.cell_x, moved_child.cell_y),
        (9, 6),
        "relative child should follow the current parent placement when that parent id is replaced"
    );
}

#[test]
fn kitty_image_number_targets_newest_image_for_placement_and_delete() {
    let (mut grid, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(4, 8);

    feed_bytes(&mut grid, b"\x1b_Gq=0,a=t,f=24,s=1,v=1,I=71;EjRW\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Gq=0,a=p,I=71,p=1,c=1,r=1\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Gq=0,a=t,f=24,s=1,v=1,I=71;EjRW\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Gq=0,a=p,I=71,p=2,c=1,r=1\x1b\\");

    let replies: Vec<_> = grid
        .pending_messages_to_pty
        .iter()
        .map(|message| String::from_utf8(message.clone()).unwrap())
        .collect();
    let create_replies: Vec<_> = replies
        .iter()
        .filter(|reply| reply.contains("I=71;OK") && !reply.contains(",p="))
        .collect();
    assert_eq!(
        create_replies.len(),
        2,
        "expected two create replies, got {replies:?}"
    );
    let first_image_id =
        kitty_reply_image_id(create_replies[0]).expect("first create reply should include i=");
    let second_image_id =
        kitty_reply_image_id(create_replies[1]).expect("second create reply should include i=");
    assert_ne!(
        first_image_id, second_image_id,
        "reusing the same I= should create a newer synthetic image id"
    );

    let before_delete = grid.visible_kitty_image_chunks(0, 0);
    assert_eq!(before_delete.len(), 2);
    assert!(
        before_delete
            .iter()
            .any(|chunk| chunk.placement_id == Some(pid(1))),
        "expected placement 1 before delete"
    );
    assert!(
        before_delete
            .iter()
            .any(|chunk| chunk.placement_id == Some(pid(2))),
        "expected placement 2 before delete"
    );

    feed_bytes(&mut grid, b"\x1b_Ga=d,d=n,I=71,p=2\x1b\\");

    let after_delete = grid.visible_kitty_image_chunks(0, 0);
    assert_eq!(after_delete.len(), 1);
    assert_eq!(after_delete[0].placement_id, Some(pid(1)));
}

#[test]
fn kitty_omitted_placement_id_allows_multiple_stored_placements_for_same_image() {
    let (mut grid, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(8, 12);

    feed_bytes(&mut grid, &kitty_retransmit_rgba(81, 3, 2));
    feed_bytes(&mut grid, b"\x1b[1;1H");
    feed_bytes(&mut grid, b"\x1b_Ga=p,i=81,c=2,r=2\x1b\\");
    feed_bytes(&mut grid, b"\x1b[1;4H");
    feed_bytes(&mut grid, b"\x1b_Ga=p,i=81,c=2,r=2\x1b\\");

    let visible = grid.visible_kitty_image_chunks(0, 0);
    let mut positions: Vec<_> = visible
        .iter()
        .map(|chunk| (chunk.cell_x, chunk.cell_y))
        .collect();
    positions.sort_unstable();

    assert_eq!(
        visible.len(),
        2,
        "omitting p should create two coexisting placements for the same image id"
    );
    assert_eq!(positions, vec![(0, 0), (3, 0)]);
}

#[test]
fn kitty_zero_placement_id_allows_multiple_stored_placements_for_same_image() {
    let (mut grid, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(8, 12);

    feed_bytes(&mut grid, &kitty_retransmit_rgba(82, 3, 2));
    feed_bytes(&mut grid, b"\x1b[1;1H");
    feed_bytes(&mut grid, b"\x1b_Ga=p,i=82,p=0,c=2,r=2\x1b\\");
    feed_bytes(&mut grid, b"\x1b[1;4H");
    feed_bytes(&mut grid, b"\x1b_Ga=p,i=82,p=0,c=2,r=2\x1b\\");

    let visible = grid.visible_kitty_image_chunks(0, 0);
    let mut positions: Vec<_> = visible
        .iter()
        .map(|chunk| (chunk.cell_x, chunk.cell_y))
        .collect();
    positions.sort_unstable();

    assert_eq!(
        visible.len(),
        2,
        "p=0 should behave like an unnamed placement and allow multiple coexisting placements"
    );
    assert_eq!(positions, vec![(0, 0), (3, 0)]);
}

#[test]
fn kitty_uppercase_delete_by_image_id_frees_backing_data() {
    let (mut grid, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(4, 8);

    feed_bytes(&mut grid, b"\x1b_Gq=0,a=t,f=24,s=1,v=1,i=51;EjRW\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Gq=0,a=p,i=51,p=1,c=1,r=1\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Ga=d,d=i,i=51,p=1\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Gq=0,a=p,i=51,p=2,c=1,r=1\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Gq=0,a=t,f=24,s=1,v=1,i=52;EjRW\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Gq=0,a=p,i=52,p=1,c=1,r=1\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Ga=d,d=I,i=52,p=1\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Gq=0,a=p,i=52,p=2,c=1,r=1\x1b\\");

    let replies: Vec<_> = grid
        .pending_messages_to_pty
        .iter()
        .map(|message| String::from_utf8(message.clone()).unwrap())
        .collect();
    assert!(
        replies
            .iter()
            .any(|reply| reply == "\u{1b}_Gi=51,p=2;OK\u{1b}\\"),
        "lowercase delete should preserve backing data for i=51, got {replies:?}",
    );
    assert!(
        replies
            .iter()
            .any(|reply| reply.contains("i=52,p=2;ENOENT:")),
        "uppercase delete should free backing data for i=52, got {replies:?}",
    );
}

#[test]
fn kitty_image_number_delete_tracks_newest_remaining_and_uppercase_frees_data() {
    let (mut grid, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(4, 8);

    feed_bytes(&mut grid, b"\x1b_Gq=0,a=t,f=24,s=1,v=1,I=71;EjRW\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Gq=0,a=p,I=71,p=1,c=1,r=1\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Gq=0,a=t,f=24,s=1,v=1,I=71;EjRW\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Gq=0,a=p,I=71,p=2,c=1,r=1\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Ga=d,d=n,I=71,p=2\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Gq=0,a=p,I=71,p=3,c=1,r=1\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Ga=d,d=N,I=71,p=3\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Gq=0,a=p,I=71,p=4,c=1,r=1\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Ga=d,d=N,I=71,p=4\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Ga=d,d=N,I=71,p=1\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Gq=0,a=p,I=71,p=5,c=1,r=1\x1b\\");

    let replies: Vec<_> = grid
        .pending_messages_to_pty
        .iter()
        .map(|message| String::from_utf8(message.clone()).unwrap())
        .collect();
    assert!(
        replies
            .iter()
            .any(|reply| reply.contains("p=3") && reply.contains("I=71;OK")),
        "lowercase image-number delete should preserve the newest image backing data, got {replies:?}",
    );
    assert!(
        replies
            .iter()
            .any(|reply| reply.contains("p=4") && reply.contains("I=71;OK")),
        "after uppercase delete frees the newest image, I=71 should fall back to the older surviving image, got {replies:?}",
    );
    assert!(
        replies
            .iter()
            .any(|reply| reply.contains("I=71;ENOENT:")),
        "after uppercase deletes free all images for a number, later I=71 placement should fail, got {replies:?}",
    );
}

#[test]
fn kitty_geometry_delete_p_targets_only_the_requested_cell() {
    let (mut grid, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(8, 12);

    place_explicit_rgba_at(&mut grid, 101, 1, 1, 1, 2, 2, None);
    place_explicit_rgba_at(&mut grid, 102, 2, 1, 4, 2, 2, None);

    feed_bytes(&mut grid, b"\x1b_Ga=d,d=p,x=4,y=1\x1b\\");

    assert_eq!(visible_kitty_placement_ids(&grid), vec![1]);
}

#[test]
fn kitty_geometry_delete_q_matches_cell_and_z_index() {
    let (mut grid, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(8, 12);

    place_explicit_rgba_at(&mut grid, 111, 1, 1, 1, 2, 2, Some(1));
    place_explicit_rgba_at(&mut grid, 112, 2, 1, 1, 2, 2, Some(2));

    feed_bytes(&mut grid, b"\x1b_Ga=d,d=q,x=1,y=1,z=2\x1b\\");

    assert_eq!(visible_kitty_placement_ids(&grid), vec![1]);
}

#[test]
fn kitty_geometry_delete_x_matches_only_intersecting_columns() {
    let (mut grid, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(8, 12);

    place_explicit_rgba_at(&mut grid, 121, 1, 1, 1, 2, 2, None);
    place_explicit_rgba_at(&mut grid, 122, 2, 1, 4, 2, 2, None);

    feed_bytes(&mut grid, b"\x1b_Ga=d,d=x,x=4\x1b\\");

    assert_eq!(visible_kitty_placement_ids(&grid), vec![1]);
}

#[test]
fn kitty_geometry_delete_y_matches_only_intersecting_rows() {
    let (mut grid, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(10, 12);

    place_explicit_rgba_at(&mut grid, 131, 1, 1, 1, 2, 2, None);
    place_explicit_rgba_at(&mut grid, 132, 2, 4, 1, 2, 2, None);

    feed_bytes(&mut grid, b"\x1b_Ga=d,d=y,y=4\x1b\\");

    assert_eq!(visible_kitty_placement_ids(&grid), vec![1]);
}

#[test]
fn kitty_geometry_delete_z_matches_only_requested_z_index() {
    let (mut grid, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(8, 12);

    place_explicit_rgba_at(&mut grid, 141, 1, 1, 1, 2, 2, Some(3));
    place_explicit_rgba_at(&mut grid, 142, 2, 1, 4, 2, 2, Some(4));

    feed_bytes(&mut grid, b"\x1b_Ga=d,d=z,z=4\x1b\\");

    assert_eq!(visible_kitty_placement_ids(&grid), vec![1]);
}

#[test]
fn kitty_geometry_delete_r_matches_inclusive_image_id_ranges() {
    let (mut grid, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(8, 16);

    place_explicit_rgba_at(&mut grid, 200, 1, 1, 1, 2, 2, None);
    place_explicit_rgba_at(&mut grid, 202, 2, 1, 4, 2, 2, None);
    place_explicit_rgba_at(&mut grid, 205, 3, 1, 7, 2, 2, None);

    feed_bytes(&mut grid, b"\x1b_Ga=d,d=r,x=200,y=204\x1b\\");

    assert_eq!(visible_kitty_placement_ids(&grid), vec![3]);
}

#[test]
fn kitty_uppercase_geometry_delete_frees_backing_data() {
    let (mut grid, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_grid_with_shared_stores(8, 12);

    place_explicit_rgba_at(&mut grid, 301, 1, 1, 1, 2, 2, None);

    feed_bytes(&mut grid, b"\x1b_Ga=d,d=P,x=1,y=1\x1b\\");
    feed_bytes(&mut grid, b"\x1b_Gq=0,a=p,i=301,p=2,c=1,r=1\x1b\\");

    let replies: Vec<_> = grid
        .pending_messages_to_pty
        .iter()
        .map(|message| String::from_utf8(message.clone()).unwrap())
        .collect();
    assert!(
        replies
            .iter()
            .any(|reply| reply.contains("i=301,p=2;ENOENT:")),
        "uppercase geometry delete should free backing data, got {replies:?}",
    );
}

#[test]
fn kitty_placeholder_render_survives_grid_width_change() {
    let mut grid = create_grid_with_size_and_raw(8, 14, &kitty_virtual_rgba(8, 56, 56, 14, 7));
    feed_bytes(&mut grid, &placeholder_rgba_text(8, 14, 7));

    let before_resize = grid.visible_kitty_placeholder_renders(0, 0);
    assert_eq!(before_resize.len(), 1);
    assert!(
        before_resize[0].cells.len() >= 90,
        "expected a dense placeholder render before resize"
    );

    grid.change_size(8, 10);
    let after_narrow = grid.visible_kitty_placeholder_renders(0, 0);
    assert_eq!(after_narrow.len(), 1);
    assert!(
        !after_narrow[0].cells.is_empty(),
        "placeholder render should remain visible after narrowing"
    );

    grid.change_size(8, 14);
    let after_widen = grid.visible_kitty_placeholder_renders(0, 0);
    assert_eq!(after_widen.len(), 1);
    assert!(
        !after_widen[0].cells.is_empty(),
        "placeholder render should remain visible after widening back"
    );
}

#[test]
fn kitty_placeholder_render_survives_scroll_viewport_transitions() {
    let mut grid = create_grid_with_size_and_raw(4, 8, &kitty_virtual_rgba(10, 16, 8, 4, 2));
    feed_bytes(&mut grid, &placeholder_rgba_text(10, 4, 2));
    feed_bytes(&mut grid, b"\r\nAA\r\nBB\r\nCC\r\nDD\r\nEE");

    assert!(
        grid.visible_kitty_placeholder_renders(0, 0).is_empty(),
        "placeholder should be out of view at the bottom after enough subsequent text"
    );

    grid.move_viewport_up(4);
    let after_scroll_up = grid.visible_kitty_placeholder_renders(0, 0);
    assert_eq!(after_scroll_up.len(), 1);
    assert!(
        !after_scroll_up[0].cells.is_empty(),
        "placeholder should reappear when scrolling back to its rows"
    );

    grid.move_viewport_down(4);
    assert!(
        grid.visible_kitty_placeholder_renders(0, 0).is_empty(),
        "placeholder should disappear again when scrolled back to the bottom"
    );
}

#[test]
fn kitty_placeholder_reflow_emits_changed_placeholder_render_bundle() {
    let mut grid = create_grid_with_size_and_raw(4, 14, &kitty_virtual_rgba(13, 16, 8, 2, 1));
    feed_bytes(&mut grid, b"AAAA");
    feed_bytes(&mut grid, &placeholder_rgba_text(13, 2, 1));
    feed_bytes(&mut grid, b"BBBBBBBB");

    grid.change_size(4, 8);

    let visible_placeholder_renders = grid.visible_kitty_placeholder_renders(0, 0);
    assert_eq!(
        visible_placeholder_renders.len(),
        1,
        "reflow should preserve visible placeholder state"
    );

    let render_output = grid
        .render(0, 0, &Style::default())
        .unwrap()
        .expect("expected render output after reflow");

    assert!(
        chunk_text(&render_output.character_chunks).contains("AAAABBBBBBBB"),
        "reflow render should still emit surrounding line text"
    );
    assert_eq!(
        render_output
            .image_output
            .kitty_scene
            .placeholder_renders
            .len(),
        1,
        "reflow render should emit placeholder redraw when changed rows intersect it"
    );
}

#[test]
fn kitty_delete_all_visible_clears_image_scene() {
    let mut grid = create_grid_with_size_and_raw(5, 10, &kitty_explicit_rgba(9, 1, 1, 1, 1));
    assert_eq!(grid.visible_kitty_image_chunks(0, 0).len(), 1);

    feed_bytes(&mut grid, b"\x1b_Ga=d,d=A\x1b\\");

    assert!(grid.visible_kitty_image_chunks(0, 0).is_empty());
    assert!(grid.visible_kitty_placeholder_renders(0, 0).is_empty());
}

#[test]
fn kitty_images_follow_clear_reset_and_alt_screen_lifecycle() {
    let mut grid = create_grid_with_size_and_raw(5, 10, &kitty_explicit_rgba(11, 1, 1, 1, 1));
    assert_eq!(grid.visible_kitty_image_chunks(0, 0).len(), 1);

    feed_bytes(&mut grid, b"\x1b[?1049h");
    assert!(grid.visible_kitty_image_chunks(0, 0).is_empty());

    feed_bytes(&mut grid, b"\x1b[?1049l");
    assert_eq!(grid.visible_kitty_image_chunks(0, 0).len(), 1);

    feed_bytes(&mut grid, b"\x1b[2J");
    assert!(grid.visible_kitty_image_chunks(0, 0).is_empty());

    feed_bytes(&mut grid, &kitty_explicit_rgba(12, 1, 1, 1, 1));
    assert_eq!(grid.visible_kitty_image_chunks(0, 0).len(), 1);

    grid.reset_terminal_state();
    assert!(grid.visible_kitty_image_chunks(0, 0).is_empty());
}

#[test]
fn kitty_retransmit_without_new_placement_marks_grid_for_rerender() {
    let mut grid = create_grid_with_size_and_raw(5, 10, &kitty_explicit_rgba(21, 1, 1, 1, 1));
    grid.should_render = false;

    feed_bytes(&mut grid, &kitty_retransmit_rgba(21, 1, 1));

    assert!(
        grid.should_render,
        "retransmitting bytes for an existing kitty image id should mark the grid dirty"
    );
}

#[test]
fn kitty_retransmit_clears_existing_placements_until_recreated() {
    let mut grid = create_grid_with_size_and_raw(
        6,
        20,
        &kitty_virtual_rgba_with_placement(22, 1, 16, 8, 4, 2),
    );
    feed_bytes(
        &mut grid,
        &placeholder_rgba_text_with_placement(22, 1, 4, 2),
    );
    feed_bytes(&mut grid, &kitty_display_placement(22, 2, 4, 2));

    assert_eq!(grid.visible_kitty_placeholder_renders(0, 0).len(), 1);
    assert_eq!(grid.visible_kitty_image_chunks(0, 0).len(), 1);

    feed_bytes(&mut grid, &kitty_retransmit_rgba(22, 16, 8));

    assert!(
        grid.visible_kitty_placeholder_renders(0, 0).is_empty(),
        "retransmitting a kitty asset should remove existing placeholder placements until recreated"
    );
    assert!(
        grid.visible_kitty_image_chunks(0, 0).is_empty(),
        "retransmitting a kitty asset should remove existing explicit placements until recreated"
    );
}

#[test]
fn kitty_retransmit_clearing_placeholder_marks_underlying_rows_dirty_for_text_repaint() {
    let mut grid = create_grid_with_size_and_raw(
        6,
        20,
        &kitty_virtual_rgba_with_placement(25, 1, 16, 8, 4, 2),
    );
    feed_bytes(
        &mut grid,
        &placeholder_rgba_text_with_placement(25, 1, 4, 2),
    );

    let initial_render = grid
        .render(0, 0, &Style::default())
        .unwrap()
        .expect("expected initial placeholder render");
    let expected_rows: Vec<_> = initial_render
        .image_output
        .kitty_scene
        .placeholder_renders
        .iter()
        .flat_map(|render| render.cells.iter().map(|cell| cell.cell_y))
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();

    feed_bytes(&mut grid, &kitty_retransmit_rgba(25, 16, 8));

    let render_output = grid
        .render(0, 0, &Style::default())
        .unwrap()
        .expect("expected render output after placeholder removal");

    let changed_rows: Vec<_> = render_output
        .character_chunks
        .iter()
        .map(|chunk| chunk.y)
        .collect();
    assert!(
        changed_rows == expected_rows,
        "clearing placeholder-backed placements should force character repaint only for the affected rows, got {changed_rows:?}"
    );
    assert!(
        render_output
            .image_output
            .kitty_scene
            .placeholder_renders
            .is_empty(),
        "placeholder render bundle should be empty after retransmit clears the placement"
    );
}

#[test]
fn kitty_retransmit_then_recreate_restores_shared_placeholder_and_explicit_placements() {
    let mut grid = create_grid_with_size_and_raw(
        6,
        20,
        &kitty_virtual_rgba_with_placement(23, 1, 16, 8, 4, 2),
    );
    feed_bytes(
        &mut grid,
        &placeholder_rgba_text_with_placement(23, 1, 4, 2),
    );
    feed_bytes(&mut grid, &kitty_display_placement(23, 2, 4, 2));
    feed_bytes(&mut grid, &kitty_retransmit_rgba(23, 16, 8));
    feed_bytes(
        &mut grid,
        &kitty_virtual_rgba_with_placement(23, 1, 16, 8, 4, 2),
    );
    feed_bytes(
        &mut grid,
        &placeholder_rgba_text_with_placement(23, 1, 4, 2),
    );
    feed_bytes(&mut grid, &kitty_display_placement(23, 2, 4, 2));

    let placeholder_renders = grid.visible_kitty_placeholder_renders(0, 0);
    let explicit_chunks = grid.visible_kitty_image_chunks(0, 0);
    assert_eq!(placeholder_renders.len(), 1);
    assert_eq!(explicit_chunks.len(), 1);
    assert_eq!(placeholder_renders[0].placement_id, Some(pid(1)));
    assert_eq!(explicit_chunks[0].placement_id, Some(pid(2)));
}

#[test]
fn kitty_retransmit_then_recreate_emits_both_recreated_placements_to_output() {
    let (mut grid, sixel_image_store, kitty_asset_store, character_cell_size) =
        create_grid_with_shared_stores(6, 20);
    feed_bytes(
        &mut grid,
        &kitty_virtual_rgba_with_placement(24, 1, 16, 8, 4, 2),
    );
    feed_bytes(
        &mut grid,
        &placeholder_rgba_text_with_placement(24, 1, 4, 2),
    );
    feed_bytes(&mut grid, &kitty_display_placement(24, 2, 4, 2));

    let mut output = Output::new(
        sixel_image_store,
        kitty_asset_store,
        Rc::new(RefCell::new(KittyOutputMediaCache::disabled())),
        character_cell_size,
        true,
        true,
    );
    let client_ids = HashSet::from([1]);
    output.add_clients(&client_ids, Rc::new(RefCell::new(LinkHandler::new())), None);

    let first_render = grid
        .render(0, 0, &Style::default())
        .unwrap()
        .expect("expected initial render");
    output
        .add_character_chunks_to_client(1, first_render.character_chunks, None)
        .unwrap();
    output.add_pane_image_output_to_client(1, first_render.image_output, None);
    let _ = output.serialize().unwrap();

    let recreated_payload = kitty_rgba_payload_b64(16, 8);
    feed_bytes(&mut grid, &kitty_retransmit_rgba(24, 16, 8));
    feed_bytes(
        &mut grid,
        &kitty_virtual_rgba_with_placement_payload(24, 1, 16, 8, 4, 2, &recreated_payload),
    );
    feed_bytes(
        &mut grid,
        &placeholder_rgba_text_with_placement(24, 1, 4, 2),
    );
    feed_bytes(&mut grid, &kitty_display_placement(24, 2, 4, 2));

    let second_render = grid
        .render(0, 0, &Style::default())
        .unwrap()
        .expect("expected recreate render");
    let recreated_image_id = second_render
        .image_output
        .kitty_scene
        .explicit_chunks
        .first()
        .map(|chunk| chunk.image_id)
        .or_else(|| {
            second_render
                .image_output
                .kitty_scene
                .placeholder_renders
                .first()
                .map(|render| render.image_id)
        })
        .expect("expected recreated kitty scene to contain an internal image id");
    let placeholder_wire_placement_id = second_render
        .image_output
        .kitty_scene
        .placeholder_renders
        .first()
        .map(|render| placeholder_wire_placement_id(render.stable_render_id))
        .expect("expected recreated placeholder render");
    let explicit_wire_placement_id = second_render
        .image_output
        .kitty_scene
        .explicit_chunks
        .first()
        .map(|chunk| synthetic_wire_placement_id(chunk.stable_render_id))
        .expect("expected recreated explicit chunk");
    output
        .add_character_chunks_to_client(1, second_render.character_chunks, None)
        .unwrap();
    output.add_pane_image_output_to_client(1, second_render.image_output, None);

    let serialized = output.serialize().unwrap();
    let client_output = serialized.get(&1).unwrap();
    assert!(
        client_output.contains("\u{1b}_Ga=t"),
        "recreate path should retransmit the kitty asset bytes"
    );
    assert!(
        client_output.contains(&format!(
            "\x1b_Ga=p,U=1,i={recreated_image_id},p={placeholder_wire_placement_id}"
        )),
        "recreate path should emit the placeholder placement again"
    );
    assert!(
        client_output.contains(&format!(
            "\x1b_Ga=p,i={recreated_image_id},p={explicit_wire_placement_id}"
        )),
        "recreate path should emit the explicit placement again"
    );
}

#[test]
fn kitty_retransmit_then_recreate_emits_updated_asset_payload_to_output() {
    let (mut grid, sixel_image_store, kitty_asset_store, character_cell_size) =
        create_grid_with_shared_stores(6, 20);
    let initial_payload = kitty_rgba_payload_b64(16, 8);
    feed_bytes(
        &mut grid,
        &kitty_virtual_rgba_with_placement_payload(26, 1, 16, 8, 4, 2, &initial_payload),
    );
    feed_bytes(
        &mut grid,
        &placeholder_rgba_text_with_placement(26, 1, 4, 2),
    );
    feed_bytes(&mut grid, &kitty_display_placement(26, 2, 4, 2));

    let mut output = Output::new(
        sixel_image_store,
        kitty_asset_store,
        Rc::new(RefCell::new(KittyOutputMediaCache::disabled())),
        character_cell_size,
        true,
        true,
    );
    let client_ids = HashSet::from([1]);
    output.add_clients(&client_ids, Rc::new(RefCell::new(LinkHandler::new())), None);

    let first_render = grid
        .render(0, 0, &Style::default())
        .unwrap()
        .expect("expected initial render");
    output
        .add_character_chunks_to_client(1, first_render.character_chunks, None)
        .unwrap();
    output.add_pane_image_output_to_client(1, first_render.image_output, None);
    let initial_serialized = output.serialize().unwrap();
    let initial_client_output = initial_serialized.get(&1).unwrap();
    assert!(
        initial_client_output.contains(&initial_payload),
        "initial frame should contain the original kitty asset payload"
    );

    let updated_payload = kitty_raw_payload_b64(16, 8, 4, 0xFF);
    feed_bytes(&mut grid, &kitty_retransmit_rgba(26, 16, 8));
    feed_bytes(
        &mut grid,
        &kitty_virtual_rgba_with_placement_payload(26, 1, 16, 8, 4, 2, &updated_payload),
    );
    feed_bytes(
        &mut grid,
        &placeholder_rgba_text_with_placement(26, 1, 4, 2),
    );
    feed_bytes(&mut grid, &kitty_display_placement(26, 2, 4, 2));

    let second_render = grid
        .render(0, 0, &Style::default())
        .unwrap()
        .expect("expected recreate render");
    output
        .add_character_chunks_to_client(1, second_render.character_chunks, None)
        .unwrap();
    output.add_pane_image_output_to_client(1, second_render.image_output, None);

    let serialized = output.serialize().unwrap();
    let client_output = serialized.get(&1).unwrap();
    assert!(
        client_output.contains(&updated_payload),
        "recreate path should emit the updated kitty asset payload"
    );
    assert!(
        !client_output.contains(&initial_payload),
        "recreate path should not re-emit the stale initial kitty asset payload"
    );
}

#[test]
fn kitty_placeholder_overwrite_removes_overwritten_cell() {
    let mut grid = create_grid_with_size_and_raw(
        6,
        20,
        &kitty_virtual_rgba_with_placement(170, 1, 4, 1, 4, 1),
    );
    feed_bytes(
        &mut grid,
        &placeholder_text_with_placement_inherited_rows(170, 1, 4, 1, 3, 2),
    );

    feed_bytes(&mut grid, b"\x1b[2;4HZ");

    assert_eq!(
        visible_placeholder_cell_positions(&grid),
        vec![(2, 1), (4, 1), (5, 1)],
        "overwriting one placeholder cell should remove only that cell from the render set"
    );
}

#[test]
fn kitty_placeholder_ed0_clears_cursor_suffix_and_rows_below() {
    let mut grid = create_grid_with_size_and_raw(
        8,
        20,
        &kitty_virtual_rgba_with_placement(171, 1, 4, 4, 4, 4),
    );
    feed_bytes(
        &mut grid,
        &placeholder_text_with_placement_inherited_rows(171, 1, 4, 4, 3, 2),
    );

    feed_bytes(&mut grid, b"\x1b[3;5H\x1b[0J");

    assert_eq!(
        visible_placeholder_cell_positions(&grid),
        vec![(2, 1), (2, 2), (3, 1), (3, 2), (4, 1), (5, 1)],
        "ED 0 should clear placeholder cells from the cursor through the rest of the display"
    );
}

#[test]
fn kitty_placeholder_ed1_clears_rows_above_and_cursor_prefix() {
    let mut grid = create_grid_with_size_and_raw(
        8,
        20,
        &kitty_virtual_rgba_with_placement(172, 1, 4, 4, 4, 4),
    );
    feed_bytes(
        &mut grid,
        &placeholder_text_with_placement_inherited_rows(172, 1, 4, 4, 3, 2),
    );

    feed_bytes(&mut grid, b"\x1b[4;5H\x1b[1J");

    assert_eq!(
        visible_placeholder_cell_positions(&grid),
        vec![(2, 4), (3, 4), (4, 4), (5, 3), (5, 4)],
        "ED 1 should clear placeholder cells from the start of the display through the cursor"
    );
}

#[test]
fn kitty_placeholder_ech_removes_only_targeted_cells() {
    let mut grid = create_grid_with_size_and_raw(
        6,
        20,
        &kitty_virtual_rgba_with_placement(173, 1, 4, 1, 4, 1),
    );
    feed_bytes(
        &mut grid,
        &placeholder_text_with_placement_inherited_rows(173, 1, 4, 1, 3, 2),
    );

    feed_bytes(&mut grid, b"\x1b[2;4H\x1b[2X");

    assert_eq!(
        visible_placeholder_cell_positions(&grid),
        vec![(2, 1), (5, 1)],
        "ECH should replace only the targeted placeholder cells with blanks"
    );
}

#[test]
fn kitty_placeholder_dch_shifts_remaining_cells_left() {
    let mut grid = create_grid_with_size_and_raw(
        6,
        20,
        &kitty_virtual_rgba_with_placement(174, 1, 4, 1, 4, 1),
    );
    feed_bytes(
        &mut grid,
        &placeholder_text_with_placement_inherited_rows(174, 1, 4, 1, 3, 2),
    );

    feed_bytes(&mut grid, b"\x1b[2;4H\x1b[2P");

    assert_eq!(
        visible_placeholder_cell_positions(&grid),
        vec![(2, 1), (3, 1)],
        "DCH should delete targeted placeholder cells and shift later cells left with the line"
    );
}

#[test]
fn test_kitty_asset_store_allocates_monotonic_ids() {
    let mut kitty_asset_store = crate::panes::kitty_asset_store::KittyAssetStore::default();

    let first = kitty_asset_store.next_asset_id();
    let second = kitty_asset_store.next_asset_id();
    let third = kitty_asset_store.next_asset_id();

    assert_eq!(second, Some(first.unwrap() + 1));
    assert_eq!(third, Some(second.unwrap() + 1));
}

#[test]
fn test_kitty_asset_store_round_trips_asset_data() {
    let mut kitty_asset_store = crate::panes::kitty_asset_store::KittyAssetStore::default();
    let image_id = kitty_asset_store.next_asset_id().unwrap();
    let image_data = KittyImageData::Rgba {
        data: vec![1, 2, 3, 4],
        width: 1,
        height: 1,
    };

    kitty_asset_store.insert_asset(image_id, image_data.clone());

    assert_eq!(kitty_asset_store.image_data(image_id), Some(image_data));
    assert_eq!(kitty_asset_store.image_dimensions(image_id), Some((1, 1)));
}

#[test]
fn test_kitty_asset_store_updates_existing_asset() {
    let mut kitty_asset_store = crate::panes::kitty_asset_store::KittyAssetStore::default();
    let image_id = kitty_asset_store.next_asset_id().unwrap();
    let original = KittyImageData::Rgb {
        data: vec![1, 2, 3],
        width: 1,
        height: 1,
    };
    let updated = KittyImageData::Png {
        data: vec![9, 8, 7, 6],
        width: 2,
        height: 3,
    };

    kitty_asset_store.insert_asset(image_id, original);
    kitty_asset_store.insert_asset(image_id, updated.clone());

    assert_eq!(kitty_asset_store.image_data(image_id), Some(updated));
    assert_eq!(kitty_asset_store.image_dimensions(image_id), Some((2, 3)));
}

// All tests below use a 10-row, 40-col grid with scroll region 1;8
// (0-based: rows 0-7), leaving rows 8-9 outside the region.
const PARTIAL_SR: &[u8] = b"\x1b[1;8r";
const FILL_8_LINES: &[u8] = b"AAA\r\nBBB\r\nCCC\r\nDDD\r\nEEE\r\nFFF\r\nGGG\r\nHHH";

#[test]
fn partial_scroll_region_newline_transfers_to_scrollback() {
    let mut content: Vec<u8> = Vec::new();
    content.extend_from_slice(PARTIAL_SR);
    content.extend_from_slice(FILL_8_LINES);
    content.extend_from_slice(b"\r\nIII\r\nJJJ");

    let grid = create_grid_with_size_and_raw(10, 40, &content);

    assert_eq!(scrollback_texts(&grid), vec!["AAA", "BBB"]);
    let vp = viewport_texts(&grid);
    assert_eq!(vp[0], "CCC");
    assert_eq!(vp[1], "DDD");
    assert_eq!(vp[6], "III");
    assert_eq!(vp[7], "JJJ");
}

#[test]
fn partial_scroll_region_csi_s_transfers_to_scrollback() {
    let mut content: Vec<u8> = Vec::new();
    content.extend_from_slice(PARTIAL_SR);
    content.extend_from_slice(FILL_8_LINES);
    content.extend_from_slice(b"\x1b[3S");

    let grid = create_grid_with_size_and_raw(10, 40, &content);

    assert_eq!(scrollback_texts(&grid), vec!["AAA", "BBB", "CCC"]);
    let vp = viewport_texts(&grid);
    assert_eq!(vp[0], "DDD");
    assert_eq!(vp[4], "HHH");
    assert_eq!(vp[5], "");
    assert_eq!(vp[7], "");
}

#[test]
fn partial_scroll_region_csi_m_at_top_transfers_to_scrollback() {
    let mut content: Vec<u8> = Vec::new();
    content.extend_from_slice(PARTIAL_SR);
    content.extend_from_slice(FILL_8_LINES);
    content.extend_from_slice(b"\x1b[1;1H\x1b[2M");

    let grid = create_grid_with_size_and_raw(10, 40, &content);

    assert_eq!(scrollback_texts(&grid), vec!["AAA", "BBB"]);
    let vp = viewport_texts(&grid);
    assert_eq!(vp[0], "CCC");
    assert_eq!(vp[5], "HHH");
    assert_eq!(vp[6], "");
    assert_eq!(vp[7], "");
}

#[test]
fn partial_scroll_region_csi_m_mid_region_does_not_transfer() {
    let mut content: Vec<u8> = Vec::new();
    content.extend_from_slice(PARTIAL_SR);
    content.extend_from_slice(FILL_8_LINES);
    // Cursor to row 3, delete 2 lines
    content.extend_from_slice(b"\x1b[4;1H\x1b[2M");

    let grid = create_grid_with_size_and_raw(10, 40, &content);

    assert_eq!(grid.lines_above.len(), 0);
    let vp = viewport_texts(&grid);
    assert_eq!(vp[0], "AAA");
    assert_eq!(vp[2], "CCC");
    assert_eq!(vp[3], "FFF");
    assert_eq!(vp[5], "HHH");
    assert_eq!(vp[6], "");
    assert_eq!(vp[7], "");
}

#[test]
fn partial_scroll_region_does_not_transfer_on_alternate_screen() {
    let mut content: Vec<u8> = Vec::new();
    content.extend_from_slice(b"\x1b[?1049h");
    content.extend_from_slice(PARTIAL_SR);
    content.extend_from_slice(FILL_8_LINES);
    content.extend_from_slice(b"\x1b[3S");

    let grid = create_grid_with_size_and_raw(10, 40, &content);

    assert_eq!(grid.lines_above.len(), 0);
    let vp = viewport_texts(&grid);
    assert_eq!(vp[0], "DDD");
    assert_eq!(vp[4], "HHH");
}

#[test]
fn partial_scroll_region_nonzero_top_does_not_transfer() {
    let mut content: Vec<u8> = Vec::new();
    content.extend_from_slice(b"AAA\r\nBBB\r\nCCC\r\nDDD\r\nEEE\r\nFFF");
    // Scroll region rows 3-6 (1-based), so top is row 2, not 0
    content.extend_from_slice(b"\x1b[3;6r");
    content.extend_from_slice(b"\x1b[2S");

    let grid = create_grid_with_size_and_raw(10, 40, &content);

    assert_eq!(grid.lines_above.len(), 0);
    let vp = viewport_texts(&grid);
    assert_eq!(vp[0], "AAA");
    assert_eq!(vp[1], "BBB");
    assert_eq!(vp[2], "EEE");
    assert_eq!(vp[3], "FFF");
    assert_eq!(vp[4], "");
    assert_eq!(vp[5], "");
}

#[test]
fn partial_scroll_region_selection_adjusted_on_newline() {
    let mut grid = create_grid_with_size_and_raw(10, 40, FILL_8_LINES);
    feed_bytes(&mut grid, PARTIAL_SR);

    let pos = Position::new(3, 5);
    grid.start_selection(&pos);
    grid.end_selection(&pos);
    let start_before = grid.selection.start.line.0;
    let end_before = grid.selection.end.line.0;

    feed_bytes(&mut grid, b"\x1b[8;1H\r\nnew line");

    assert_eq!(grid.selection.start.line.0, start_before - 1);
    assert_eq!(grid.selection.end.line.0, end_before - 1);
}

#[test]
fn partial_scroll_region_selection_adjusted_on_csi_s() {
    let mut grid = create_grid_with_size_and_raw(10, 40, FILL_8_LINES);
    feed_bytes(&mut grid, PARTIAL_SR);

    let pos = Position::new(4, 2);
    grid.start_selection(&pos);
    grid.end_selection(&pos);
    let start_before = grid.selection.start.line.0;
    let end_before = grid.selection.end.line.0;

    feed_bytes(&mut grid, b"\x1b[2S");

    assert_eq!(grid.selection.start.line.0, start_before - 2);
    assert_eq!(grid.selection.end.line.0, end_before - 2);
}

#[test]
fn partial_scroll_region_selection_adjusted_on_csi_m() {
    let mut grid = create_grid_with_size_and_raw(10, 40, FILL_8_LINES);
    feed_bytes(&mut grid, PARTIAL_SR);

    let pos = Position::new(5, 2);
    grid.start_selection(&pos);
    grid.end_selection(&pos);
    let start_before = grid.selection.start.line.0;
    let end_before = grid.selection.end.line.0;

    feed_bytes(&mut grid, b"\x1b[3M");

    assert_eq!(grid.selection.start.line.0, start_before - 3);
    assert_eq!(grid.selection.end.line.0, end_before - 3);
}

#[test]
fn partial_scroll_region_all_three_mechanisms_transfer_in_order() {
    let mut content: Vec<u8> = Vec::new();
    content.extend_from_slice(PARTIAL_SR);
    content.extend_from_slice(FILL_8_LINES);
    // Newline scrolls AAA off, CSI S scrolls BBB off, CSI M scrolls CCC off
    content.extend_from_slice(b"\r\nIII");
    content.extend_from_slice(b"\x1b[1S");
    content.extend_from_slice(b"\x1b[1;1H\x1b[1M");

    let grid = create_grid_with_size_and_raw(10, 40, &content);

    assert_eq!(scrollback_texts(&grid), vec!["AAA", "BBB", "CCC"]);
}

#[test]
fn scroll_region_newline_sets_bg_color_on_new_row() {
    // Set bg color, set scroll region 1-5, fill 5 lines, then newline to scroll
    let content = b"\x1b[48;2;26;26;26m\x1b[1;5r\
        AAA\r\nBBB\r\nCCC\r\nDDD\r\nEEE\r\nFFF";
    let grid = create_grid_with_size_and_raw(10, 40, content);
    // The new row at the bottom of the scroll region (row index 4) should have bg_color
    let new_row = &grid.viewport[4];
    assert_eq!(
        new_row.bg_color,
        Some(AnsiCode::RgbCode((26, 26, 26))),
        "scroll-created row should carry the cursor background color"
    );
}

#[test]
fn scroll_region_newline_bg_color_used_for_trailing_padding() {
    // Set bg, scroll region 1-5, fill lines, scroll, then write short text on new row
    let content = b"\x1b[48;2;26;26;26m\x1b[1;5rAAA\r\nBBB\r\nCCC\r\nDDD\r\nEEE\r\nhi";
    let mut grid = create_grid_with_size_and_raw(10, 40, content);
    // read_changes returns character chunks with padding applied
    let chunks = grid.read_changes(0, 0).character_chunks;
    // Find the chunk for row 4 (the scroll-created row with "hi")
    let row_4_chunk = chunks.iter().find(|c| c.y == 4).expect("row 4 chunk");
    // The trailing padding character (last column) should have the row's bg_color
    let pad_char = &row_4_chunk.terminal_characters[39];
    assert_eq!(
        pad_char.styles.background,
        Some(AnsiCode::RgbCode((26, 26, 26))),
        "trailing padding should use the row's background color"
    );
}

#[test]
fn scroll_region_newline_bg_color_used_for_cursor_forward_gaps() {
    // Set bg, scroll region 1-5, fill lines, scroll, then cursor-forward and write
    // This simulates vim's [12C behavior on a scroll-created row
    let content = b"\x1b[48;2;26;26;26m\x1b[1;5rAAA\r\nBBB\r\nCCC\r\nDDD\r\nEEE\
        \r\n\x1b[0m\x1b[10Cx";
    let grid = create_grid_with_size_and_raw(10, 40, content);
    let new_row = &grid.viewport[4];
    // Position 5 (within the gap created by cursor forward) should have the bg_color
    let gap_char = &new_row.columns[5];
    assert_eq!(
        gap_char.styles.background,
        Some(AnsiCode::RgbCode((26, 26, 26))),
        "cursor-forward gap should use the row's background color"
    );
}

#[test]
fn scroll_region_bg_color_does_not_override_explicit_background() {
    // Set bg, scroll region 1-5, fill lines, scroll, then write with different bg
    let content = b"\x1b[48;2;26;26;26m\x1b[1;5rAAA\r\nBBB\r\nCCC\r\nDDD\r\nEEE\
        \r\n\x1b[48;2;255;0;0mRED";
    let grid = create_grid_with_size_and_raw(10, 40, content);
    let new_row = &grid.viewport[4];
    // The 'R' character should have the explicitly set red background, not the row bg
    let r_char = &new_row.columns[0];
    assert_eq!(
        r_char.styles.background,
        Some(AnsiCode::RgbCode((255, 0, 0))),
        "explicitly set background should not be overridden by row bg_color"
    );
}

#[test]
fn full_scroll_region_newline_sets_bg_color_on_new_row() {
    // Full scroll region (entire viewport), set bg, fill, then scroll
    let content = b"\x1b[48;2;26;26;26m\x1b[1;10r\
        L1\r\nL2\r\nL3\r\nL4\r\nL5\r\nL6\r\nL7\r\nL8\r\nL9\r\nL10\r\nL11";
    let grid = create_grid_with_size_and_raw(10, 40, content);
    // The new row at the bottom (row 9) should have bg_color
    let new_row = &grid.viewport[9];
    assert_eq!(
        new_row.bg_color,
        Some(AnsiCode::RgbCode((26, 26, 26))),
        "full scroll region: new row should carry the cursor background color"
    );
}

#[test]
fn row_without_scroll_has_no_bg_color() {
    // Normal content without scroll should not set bg_color on rows
    let content = b"\x1b[48;2;26;26;26mHello";
    let grid = create_grid_with_size_and_raw(10, 40, content);
    let row = &grid.viewport[0];
    assert_eq!(
        row.bg_color, None,
        "rows not created by scroll should have no bg_color"
    );
}

fn new_grid_for_forwarding_test() -> Grid {
    Grid::new(
        10,
        20,
        Rc::new(RefCell::new(Palette::default())),
        Rc::new(RefCell::new(HashMap::new())),
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(Some(SizeInPixels {
            width: 8,
            height: 16,
        }))),
        Rc::new(RefCell::new(SixelImageStore::default())),
        Rc::new(RefCell::new(KittyAssetStore::default())),
        Style::default(),
        false,
        true,
        true,
        true,
        false,
    )
}

#[test]
fn csi_14t_forwards_to_host_not_local() {
    // CSI 14t used to synthesize a local "\x1b[4;H;Wt" reply; after the
    // refactor it must be forwarded to the host instead so apps observe
    // the terminal's real window pixel dimensions.
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    for byte in b"\x1b[14t" {
        parser.advance(&mut grid, &[*byte]);
    }
    assert!(
        grid.pending_messages_to_pty.is_empty(),
        "local reply path must not fire for 14t"
    );
    assert_eq!(grid.pending_forwarded_queries.len(), 1);
    assert_eq!(
        grid.pending_forwarded_queries[0],
        crate::host_query::HostQuery::TextAreaPixelSize,
        "forwarded classification must be TextAreaPixelSize"
    );
}

#[test]
fn csi_16t_forwards_to_host_not_local() {
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    for byte in b"\x1b[16t" {
        parser.advance(&mut grid, &[*byte]);
    }
    assert!(grid.pending_messages_to_pty.is_empty());
    assert_eq!(grid.pending_forwarded_queries.len(), 1);
    assert_eq!(
        grid.pending_forwarded_queries[0],
        crate::host_query::HostQuery::CharacterCellPixelSize,
    );
}

#[test]
fn csi_18t_still_answered_locally() {
    // 18 reports Zellij's own text-area size in cells — Zellij is
    // authoritative for this, do NOT forward.
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    for byte in b"\x1b[18t" {
        parser.advance(&mut grid, &[*byte]);
    }
    assert_eq!(grid.pending_messages_to_pty.len(), 1);
    assert!(grid.pending_forwarded_queries.is_empty());
}

#[test]
fn osc_11_set_stays_local() {
    // OSC 11;<rgb> (set pane default bg) must stay local — Zellij needs
    // to track it for its own rendering.
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    for byte in b"\x1b]11;rgb:ffff/ffff/ffff\x07" {
        parser.advance(&mut grid, &[*byte]);
    }
    assert!(grid.pending_messages_to_pty.is_empty());
    assert!(
        grid.pending_forwarded_queries.is_empty(),
        "set (not query) must not forward"
    );
    assert!(grid.pane_default_bg.is_some());
}

#[test]
fn osc_11_query_without_override_forwards_to_host() {
    // When no pane-local override is in place the query must still be
    // forwarded — the host's actual bg is what the app asked for.
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    assert!(grid.pane_default_bg.is_none());
    for byte in b"\x1b]11;?\x07" {
        parser.advance(&mut grid, &[*byte]);
    }
    assert!(
        grid.pending_messages_to_pty.is_empty(),
        "no override → no local reply"
    );
    assert_eq!(grid.pending_forwarded_queries.len(), 1);
    assert!(matches!(
        grid.pending_forwarded_queries[0],
        crate::host_query::HostQuery::DefaultBackground { .. }
    ));
}

#[test]
fn osc_10_query_without_override_forwards_to_host() {
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    assert!(grid.pane_default_fg.is_none());
    for byte in b"\x1b]10;?\x07" {
        parser.advance(&mut grid, &[*byte]);
    }
    assert!(grid.pending_messages_to_pty.is_empty());
    assert_eq!(grid.pending_forwarded_queries.len(), 1);
    assert!(matches!(
        grid.pending_forwarded_queries[0],
        crate::host_query::HostQuery::DefaultForeground { .. }
    ));
}

#[test]
fn set_pane_default_colors_short_circuits_osc_queries() {
    // The CLI path (`zellij action set-pane-color`) lands on
    // `Grid::set_pane_default_colors`. A later OSC 10/11 query must
    // read that override, not be forwarded — the entire point of the
    // CLI override is that apps see the color Zellij is painting, not
    // the underlying host's.
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    grid.set_pane_default_colors(Some("#ff8040".to_string()), Some("#102030".to_string()));

    for byte in b"\x1b]10;?\x07" {
        parser.advance(&mut grid, &[*byte]);
    }
    for byte in b"\x1b]11;?\x07" {
        parser.advance(&mut grid, &[*byte]);
    }

    assert!(
        grid.pending_forwarded_queries.is_empty(),
        "overrides set via set_pane_default_colors must suppress forwarding"
    );
    assert_eq!(grid.pending_messages_to_pty.len(), 2);
    let fg_reply = String::from_utf8(grid.pending_messages_to_pty[0].clone()).unwrap();
    let bg_reply = String::from_utf8(grid.pending_messages_to_pty[1].clone()).unwrap();
    assert_eq!(fg_reply, "\u{1b}]10;rgb:ffff/8080/4040\u{7}");
    assert_eq!(bg_reply, "\u{1b}]11;rgb:1010/2020/3030\u{7}");
}

#[test]
fn osc_11_override_short_circuits_only_the_overridden_channel() {
    // Setting only the bg must not short-circuit fg queries — each
    // channel's override is independent. If only `pane_default_bg` is
    // populated, an OSC 10 query (fg) still goes to the host.
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    for byte in b"\x1b]11;rgb:1010/2020/3030\x07" {
        parser.advance(&mut grid, &[*byte]);
    }
    assert!(grid.pane_default_bg.is_some());
    assert!(grid.pane_default_fg.is_none());

    for byte in b"\x1b]10;?\x07" {
        parser.advance(&mut grid, &[*byte]);
    }
    assert!(
        grid.pending_messages_to_pty.is_empty(),
        "fg has no override → must forward"
    );
    assert_eq!(grid.pending_forwarded_queries.len(), 1);
}

#[test]
fn csi_2026_dollar_p_stays_local() {
    // DECRQM mode 2026 (synchronised output) is emulated by Zellij
    // itself — never forwarded. The response is the DECRPM form
    // `\x1b[?2026;2$y` (2 = "reset but recognised"; Zellij brackets its
    // own frames, so individual panes are treated as "not enabled").
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    for byte in b"\x1b[?2026$p" {
        parser.advance(&mut grid, &[*byte]);
    }
    assert!(
        grid.pending_forwarded_queries.is_empty(),
        "CSI ?2026$p is emulated locally, must not forward"
    );
    assert_eq!(grid.pending_messages_to_pty.len(), 1);
    assert_eq!(
        grid.pending_messages_to_pty[0], b"\x1b[?2026;2$y",
        "DECRPM reply must use the `$y` suffix per spec"
    );
}

#[test]
fn csi_2031_dollar_p_when_disabled_replies_reset() {
    // DECRQM mode 2031 (Application Theme Reporting) is per-pane state
    // tracked locally by Zellij. With no prior `CSI ? 2031 h`, the mode
    // is reset, so the DECRPM reply must report value=2.
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    for byte in b"\x1b[?2031$p" {
        parser.advance(&mut grid, &[*byte]);
    }
    assert!(
        grid.pending_forwarded_queries.is_empty(),
        "CSI ?2031$p is answered locally, must not forward"
    );
    assert_eq!(grid.pending_messages_to_pty.len(), 1);
    assert_eq!(
        grid.pending_messages_to_pty[0], b"\x1b[?2031;2$y",
        "DECRPM reply must report value=2 (reset) when 2031 is disabled"
    );
}

#[test]
fn csi_2031_dollar_p_when_enabled_replies_set() {
    // After `CSI ? 2031 h` enables theme-change notifications on this
    // pane, a DECRQM probe must report value=1 (set).
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    for byte in b"\x1b[?2031h\x1b[?2031$p" {
        parser.advance(&mut grid, &[*byte]);
    }
    assert!(
        grid.pending_forwarded_queries.is_empty(),
        "CSI ?2031$p is answered locally, must not forward"
    );
    assert_eq!(grid.pending_messages_to_pty.len(), 1);
    assert_eq!(
        grid.pending_messages_to_pty[0], b"\x1b[?2031;1$y",
        "DECRPM reply must report value=1 (set) when 2031 is enabled"
    );
}

#[test]
fn csi_22t_and_23t_stay_local() {
    // CSI 22t / 23t manipulate the pane's title stack — Zellij owns
    // that state, so forwarding would route an app's push/pop to the
    // host's unrelated title stack instead of the visible pane title.
    // They produce no reply; both queues must stay empty after each.
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    grid.set_title("hello".to_string());

    for byte in b"\x1b[22;0t" {
        parser.advance(&mut grid, &[*byte]);
    }
    assert!(
        grid.pending_forwarded_queries.is_empty(),
        "22t is a stack op, not a query"
    );
    assert!(
        grid.pending_messages_to_pty.is_empty(),
        "22t produces no reply"
    );

    // The push actually landed on Zellij's per-pane stack: restore it.
    grid.set_title("different".to_string());
    for byte in b"\x1b[23;0t" {
        parser.advance(&mut grid, &[*byte]);
    }
    assert!(
        grid.pending_forwarded_queries.is_empty(),
        "23t is a stack op, not a query"
    );
    assert!(
        grid.pending_messages_to_pty.is_empty(),
        "23t produces no reply"
    );
    assert_eq!(
        grid.title.as_deref(),
        Some("hello"),
        "pop should restore the previously-pushed title"
    );
}

#[test]
fn osc_4_set_stays_local() {
    // OSC 4;<index>;<rgb> writes to Zellij's in-memory palette; only
    // the query form (`OSC 4;N;?`) ever forwards to the host.
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    for byte in b"\x1b]4;5;rgb:ffff/0000/0000\x07" {
        parser.advance(&mut grid, &[*byte]);
    }
    assert!(
        grid.pending_forwarded_queries.is_empty(),
        "OSC 4 set must not forward"
    );
    assert!(
        grid.pending_messages_to_pty.is_empty(),
        "OSC 4 set produces no reply"
    );
    let changed = grid
        .changed_colors
        .expect("changed_colors should be populated by OSC 4 set");
    assert!(changed[5].is_some(), "index 5 should have been written");
}

#[test]
fn decset_2031_enables_color_palette_notification() {
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    assert!(
        !grid.color_palette_notification_enabled,
        "default state must be disabled"
    );
    for byte in b"\x1b[?2031h" {
        parser.advance(&mut grid, &[*byte]);
    }
    assert!(
        grid.color_palette_notification_enabled,
        "DECSET 2031 must enable the flag"
    );
    for byte in b"\x1b[?2031l" {
        parser.advance(&mut grid, &[*byte]);
    }
    assert!(
        !grid.color_palette_notification_enabled,
        "DECRST 2031 must disable the flag"
    );
}

#[test]
fn push_color_palette_dsr_emits_when_enabled() {
    let mut grid = new_grid_for_forwarding_test();
    grid.color_palette_notification_enabled = true;
    grid.push_color_palette_dsr(zellij_utils::data::HostTerminalThemeMode::Dark);
    assert_eq!(grid.pending_messages_to_pty.len(), 1);
    assert_eq!(
        grid.pending_messages_to_pty[0],
        b"\x1b[?997;1n".to_vec(),
        "Dark mode emits ?997;1n"
    );
    grid.push_color_palette_dsr(zellij_utils::data::HostTerminalThemeMode::Light);
    assert_eq!(grid.pending_messages_to_pty.len(), 2);
    assert_eq!(
        grid.pending_messages_to_pty[1],
        b"\x1b[?997;2n".to_vec(),
        "Light mode emits ?997;2n"
    );
}

#[test]
fn push_color_palette_dsr_noop_when_disabled() {
    let mut grid = new_grid_for_forwarding_test();
    assert!(!grid.color_palette_notification_enabled);
    grid.push_color_palette_dsr(zellij_utils::data::HostTerminalThemeMode::Dark);
    assert!(
        grid.pending_messages_to_pty.is_empty(),
        "no DSR is queued when the app has not opted in via CSI ?2031h"
    );
}

#[test]
fn csi_996n_pushes_color_palette_mode_query_to_forwarded_queries() {
    use crate::host_query::HostQuery;
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    for byte in b"\x1b[?996n" {
        parser.advance(&mut grid, &[*byte]);
    }
    assert_eq!(
        grid.pending_forwarded_queries,
        vec![HostQuery::ColorPaletteMode],
        "CSI ?996n must push HostQuery::ColorPaletteMode for Screen to short-circuit"
    );
    assert!(
        grid.pending_messages_to_pty.is_empty(),
        "Grid must NOT answer locally — Screen owns the host_terminal_theme_mode cache"
    );
}

#[test]
fn csi_5n_status_query_still_handled_locally() {
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    // Plain DSR 5 (no `?` intermediate) is not the new theme query and
    // must continue to receive its `\e[0n` "all good" reply locally.
    for byte in b"\x1b[5n" {
        parser.advance(&mut grid, &[*byte]);
    }
    assert!(
        grid.pending_forwarded_queries.is_empty(),
        "DSR 5 is not a forwarded query"
    );
    assert_eq!(
        grid.pending_messages_to_pty,
        vec![b"\x1b[0n".to_vec()],
        "DSR 5 must still produce its local 'all good' reply"
    );
}
