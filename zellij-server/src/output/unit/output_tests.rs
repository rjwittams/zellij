use super::super::image_fragment::{
    visible_image_fragments, ImageFragment, KittyExplicitFragment, KittyPlaceholderFragment,
    SixelFragment,
};
use super::super::kitty_diff::{
    plan_kitty_scene, KittyAssetOp, KittyPlacementKey, KittyPlacementOp, KittyScenePlan,
    KittySceneState, PlannedKittyPlacement,
};
use super::super::{
    CharacterChunk, FloatingPanesStack, KittyImageChunk, KittyImageData, Output, OutputBuffer,
    PaneImageRenderOutput, SixelImageChunk,
};
use crate::panes::pane_image_scene::KittyRenderBundle;
use crate::panes::sixel::{SixelGrid, SixelImageStore};
use crate::panes::terminal_character::AnsiCode;
use crate::panes::{LinkHandler, Row, TerminalCharacter};
use crate::ClientId;
use sixel_image::SixelImage;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;
use zellij_utils::pane_size::{Dimension, PaneGeom, Size, SizeInPixels};

/// Helper to create a simple Output instance for testing
fn create_test_output() -> Output {
    let (output, _sixel_image_store, _character_cell_size) = create_test_output_with_state();
    output
}

fn create_test_output_with_state() -> (
    Output,
    Rc<RefCell<SixelImageStore>>,
    Rc<RefCell<Option<SizeInPixels>>>,
) {
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let character_cell_size = Rc::new(RefCell::new(Some(SizeInPixels {
        height: 20,
        width: 10,
    })));
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    (
        Output::new(
            sixel_image_store.clone(),
            character_cell_size.clone(),
            styled_underlines,
            osc8_hyperlinks,
        ),
        sixel_image_store,
        character_cell_size,
    )
}

/// Helper to create a simple CharacterChunk with text
fn create_character_chunk_from_str(text: &str, x: usize, y: usize) -> CharacterChunk {
    let terminal_chars: Vec<TerminalCharacter> =
        text.chars().map(|c| TerminalCharacter::new(c)).collect();
    CharacterChunk::new(terminal_chars, x, y)
}

fn create_kitty_chunk(image_id: u32, columns: usize, rows: usize) -> KittyImageChunk {
    KittyImageChunk {
        image_id,
        placement_id: Some(image_id),
        placement_mode: crate::output::KittyImagePlacementMode::Explicit,
        cell_x: 0,
        cell_y: 0,
        columns,
        rows,
        source_x: 0,
        source_y: 0,
        source_width: 10,
        source_height: 10,
        z_index: 0,
        x_offset: 0,
        y_offset: 0,
        image_data: KittyImageData::Png {
            data: vec![1, 2, 3, 4],
            width: 1,
            height: 1,
        },
    }
}

fn create_kitty_placeholder_render(image_id: u32) -> crate::output::KittyPlaceholderRender {
    crate::output::KittyPlaceholderRender {
        image_id,
        placement_id: Some(image_id),
        columns: 1,
        rows: 1,
        source_x: 0,
        source_y: 0,
        source_width: 10,
        source_height: 10,
        x_offset: 0,
        y_offset: 0,
        image_data: KittyImageData::Png {
            data: vec![1, 2, 3, 4],
            width: 1,
            height: 1,
        },
        cells: vec![crate::output::KittyPlaceholderCellRender {
            cell_x: 0,
            cell_y: 0,
            placeholder_row: 0,
            placeholder_col: 0,
        }],
    }
}

fn create_kitty_diff_explicit_placement(
    image_id: u32,
    placement_id: u32,
    columns: usize,
    rows: usize,
) -> PlannedKittyPlacement {
    let mut chunk = create_kitty_chunk(image_id, columns, rows);
    chunk.placement_id = Some(placement_id);
    PlannedKittyPlacement::Explicit {
        key: KittyPlacementKey {
            image_id,
            placement_id,
        },
        chunk,
    }
}

fn create_kitty_diff_placeholder_placement(
    image_id: u32,
    placement_id: u32,
) -> PlannedKittyPlacement {
    let mut render = create_kitty_placeholder_render(image_id);
    render.placement_id = Some(placement_id);
    PlannedKittyPlacement::Placeholder {
        key: KittyPlacementKey {
            image_id,
            placement_id,
        },
        render,
    }
}

fn create_kitty_scene_state(placements: Vec<PlannedKittyPlacement>) -> KittySceneState {
    let mut scene = KittySceneState::default();
    for placement in placements {
        scene.insert_placement(placement);
    }
    scene
}

fn seed_test_sixel_image(
    sixel_image_store: Rc<RefCell<SixelImageStore>>,
    character_cell_size: Rc<RefCell<Option<SizeInPixels>>>,
    image_id: usize,
) {
    let mut sixel_grid = SixelGrid::new(character_cell_size, sixel_image_store);
    let sixel_image = SixelImage::new(
        b"\x1bPq\n#0;2;0;0;0#1;2;100;100;0#2;2;0;100;0\n#1~~@@vv@@~~@@~~$\n#2??}}GG}}??}}??-\n#1!14@\n\x1b\n",
    )
    .unwrap();
    sixel_grid.new_sixel_image(image_id, sixel_image);
}

fn pane_image_output_with_sixels(sixel_chunks: Vec<SixelImageChunk>) -> PaneImageRenderOutput {
    PaneImageRenderOutput {
        sixel_chunks,
        ..Default::default()
    }
}

fn pane_image_output_with_kitty_scene(
    explicit_chunks: Vec<KittyImageChunk>,
) -> PaneImageRenderOutput {
    PaneImageRenderOutput {
        kitty_scene: KittyRenderBundle {
            explicit_chunks,
            placeholder_renders: vec![],
        },
        ..Default::default()
    }
}

fn pane_image_output_with_kitty_placeholder(
    placeholder_renders: Vec<crate::output::KittyPlaceholderRender>,
) -> PaneImageRenderOutput {
    PaneImageRenderOutput {
        kitty_scene: KittyRenderBundle {
            explicit_chunks: vec![],
            placeholder_renders,
        },
        ..Default::default()
    }
}

/// Helper to create test clients
fn create_test_clients(count: usize) -> HashSet<ClientId> {
    (1..=count).map(|i| i as ClientId).collect()
}

/// Helper to create PaneGeom for FloatingPanesStack tests
fn create_pane_geom(x: usize, y: usize, cols: usize, rows: usize) -> PaneGeom {
    PaneGeom {
        x,
        y,
        cols: Dimension::fixed(cols),
        rows: Dimension::fixed(rows),
        stacked: None,
        is_pinned: false,
        logical_position: None,
    }
}

#[test]
fn test_output_new() {
    let output = create_test_output();

    // Verify default state of all fields
    assert!(!output.is_dirty(), "New output should not be dirty");
    assert!(
        !output.has_rendered_assets(),
        "New output should not have rendered assets"
    );
}

#[test]
fn test_add_clients() {
    let mut output = create_test_output();
    let client_ids = create_test_clients(3);
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));

    output.add_clients(&client_ids, link_handler, None);

    // Verify that client_character_chunks has entries for all clients
    assert!(!output.is_dirty(), "Should not be dirty until chunks added");
}

#[test]
fn test_is_dirty_with_empty_output() {
    let output = create_test_output();
    assert!(!output.is_dirty(), "Empty output should not be dirty");
}

#[test]
fn test_is_dirty_with_character_chunks() {
    let mut output = create_test_output();
    let client_ids = create_test_clients(1);
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);

    let chunk = create_character_chunk_from_str("Hi", 0, 0);
    output
        .add_character_chunks_to_client(1, vec![chunk], None)
        .unwrap();

    assert!(
        output.is_dirty(),
        "Output should be dirty after adding character chunks"
    );
}

#[test]
fn test_is_dirty_with_pre_vte_instructions() {
    let mut output = create_test_output();
    let client_ids = create_test_clients(1);
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);

    output.add_pre_vte_instruction_to_client(1, "\u{1b}[?1049h");

    assert!(
        output.is_dirty(),
        "Output should be dirty after adding pre VTE instructions"
    );
}

#[test]
fn test_is_dirty_with_post_vte_instructions() {
    let mut output = create_test_output();
    let client_ids = create_test_clients(1);
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);

    output.add_post_vte_instruction_to_client(1, "\u{1b}[?25h");

    assert!(
        output.is_dirty(),
        "Output should be dirty after adding post VTE instructions"
    );
}

#[test]
fn test_is_dirty_with_sixel_chunks() {
    let mut output = create_test_output();
    let client_ids = create_test_clients(1);
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);

    let sixel_chunk = SixelImageChunk {
        cell_x: 0,
        cell_y: 0,
        sixel_image_pixel_x: 0,
        sixel_image_pixel_y: 0,
        sixel_image_pixel_width: 100,
        sixel_image_pixel_height: 100,
        sixel_image_id: 1,
    };
    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_sixels(vec![sixel_chunk]),
        None,
    );

    assert!(
        output.is_dirty(),
        "Output should be dirty after adding sixel chunks"
    );
}

#[test]
fn test_kitty_diff_ignores_unchanged_placement() {
    let assumed = create_kitty_scene_state(vec![create_kitty_diff_explicit_placement(1, 10, 2, 2)]);
    let desired = assumed.clone();

    let plan = plan_kitty_scene(&assumed, &desired);

    assert_eq!(
        plan,
        KittyScenePlan::Diff {
            asset_ops: vec![],
            placement_ops: vec![],
        }
    );
}

#[test]
fn test_kitty_diff_deletes_removed_placement() {
    let assumed = create_kitty_scene_state(vec![create_kitty_diff_explicit_placement(1, 10, 2, 2)]);
    let desired = KittySceneState::default();

    let plan = plan_kitty_scene(&assumed, &desired);

    assert_eq!(
        plan,
        KittyScenePlan::Diff {
            asset_ops: vec![],
            placement_ops: vec![KittyPlacementOp::Delete {
                key: KittyPlacementKey {
                    image_id: 1,
                    placement_id: 10,
                },
            }],
        }
    );
}

#[test]
fn test_kitty_diff_places_resident_asset_without_retransmit() {
    let mut assumed = KittySceneState::default();
    assumed.resident_assets.insert(
        1,
        KittyImageData::Png {
            data: vec![1, 2, 3, 4],
            width: 1,
            height: 1,
        },
    );
    let desired = create_kitty_scene_state(vec![create_kitty_diff_explicit_placement(1, 10, 2, 2)]);

    let plan = plan_kitty_scene(&assumed, &desired);

    assert_eq!(
        plan,
        KittyScenePlan::Diff {
            asset_ops: vec![],
            placement_ops: vec![KittyPlacementOp::PlaceExplicit {
                key: KittyPlacementKey {
                    image_id: 1,
                    placement_id: 10,
                },
                chunk: {
                    let mut chunk = create_kitty_chunk(1, 2, 2);
                    chunk.placement_id = Some(10);
                    chunk
                },
            }],
        }
    );
}

#[test]
fn test_kitty_diff_retransmits_missing_asset_before_place() {
    let assumed = KittySceneState::default();
    let desired = create_kitty_scene_state(vec![create_kitty_diff_explicit_placement(1, 10, 2, 2)]);

    let plan = plan_kitty_scene(&assumed, &desired);

    assert_eq!(
        plan,
        KittyScenePlan::Diff {
            asset_ops: vec![KittyAssetOp::EnsureResident {
                image_id: 1,
                image_data: KittyImageData::Png {
                    data: vec![1, 2, 3, 4],
                    width: 1,
                    height: 1,
                },
            }],
            placement_ops: vec![KittyPlacementOp::PlaceExplicit {
                key: KittyPlacementKey {
                    image_id: 1,
                    placement_id: 10,
                },
                chunk: {
                    let mut chunk = create_kitty_chunk(1, 2, 2);
                    chunk.placement_id = Some(10);
                    chunk
                },
            }],
        }
    );
}

#[test]
fn test_kitty_diff_replaces_geometry_change_with_delete_and_place() {
    let assumed = create_kitty_scene_state(vec![create_kitty_diff_explicit_placement(1, 10, 2, 2)]);
    let desired = create_kitty_scene_state(vec![create_kitty_diff_explicit_placement(1, 10, 3, 2)]);

    let plan = plan_kitty_scene(&assumed, &desired);

    assert_eq!(
        plan,
        KittyScenePlan::Diff {
            asset_ops: vec![],
            placement_ops: vec![
                KittyPlacementOp::Delete {
                    key: KittyPlacementKey {
                        image_id: 1,
                        placement_id: 10,
                    },
                },
                KittyPlacementOp::PlaceExplicit {
                    key: KittyPlacementKey {
                        image_id: 1,
                        placement_id: 10,
                    },
                    chunk: {
                        let mut chunk = create_kitty_chunk(1, 3, 2);
                        chunk.placement_id = Some(10);
                        chunk
                    },
                },
            ],
        }
    );
}

#[test]
fn test_kitty_diff_invalidates_all_placements_when_asset_payload_changes() {
    let assumed = create_kitty_scene_state(vec![
        create_kitty_diff_explicit_placement(1, 10, 2, 2),
        create_kitty_diff_explicit_placement(1, 11, 2, 2),
    ]);
    let mut changed_chunk = create_kitty_chunk(1, 2, 2);
    changed_chunk.placement_id = Some(10);
    changed_chunk.image_data = KittyImageData::Png {
        data: vec![9, 9, 9, 9],
        width: 1,
        height: 1,
    };
    let mut changed_chunk_two = changed_chunk.clone();
    changed_chunk_two.placement_id = Some(11);
    let desired = create_kitty_scene_state(vec![
        PlannedKittyPlacement::Explicit {
            key: KittyPlacementKey {
                image_id: 1,
                placement_id: 10,
            },
            chunk: changed_chunk.clone(),
        },
        PlannedKittyPlacement::Explicit {
            key: KittyPlacementKey {
                image_id: 1,
                placement_id: 11,
            },
            chunk: changed_chunk_two.clone(),
        },
    ]);

    let plan = plan_kitty_scene(&assumed, &desired);

    assert_eq!(
        plan,
        KittyScenePlan::Diff {
            asset_ops: vec![KittyAssetOp::EnsureResident {
                image_id: 1,
                image_data: KittyImageData::Png {
                    data: vec![9, 9, 9, 9],
                    width: 1,
                    height: 1,
                },
            }],
            placement_ops: vec![
                KittyPlacementOp::Delete {
                    key: KittyPlacementKey {
                        image_id: 1,
                        placement_id: 10,
                    },
                },
                KittyPlacementOp::Delete {
                    key: KittyPlacementKey {
                        image_id: 1,
                        placement_id: 11,
                    },
                },
                KittyPlacementOp::PlaceExplicit {
                    key: KittyPlacementKey {
                        image_id: 1,
                        placement_id: 10,
                    },
                    chunk: changed_chunk,
                },
                KittyPlacementOp::PlaceExplicit {
                    key: KittyPlacementKey {
                        image_id: 1,
                        placement_id: 11,
                    },
                    chunk: changed_chunk_two,
                },
            ],
        }
    );
}

#[test]
fn test_kitty_diff_shared_asset_updates_explicit_and_placeholder_placements() {
    let assumed = create_kitty_scene_state(vec![
        create_kitty_diff_explicit_placement(1, 10, 2, 2),
        create_kitty_diff_placeholder_placement(1, 20),
    ]);
    let mut changed_chunk = create_kitty_chunk(1, 2, 2);
    changed_chunk.placement_id = Some(10);
    changed_chunk.image_data = KittyImageData::Png {
        data: vec![8, 8, 8, 8],
        width: 1,
        height: 1,
    };
    let mut changed_render = create_kitty_placeholder_render(1);
    changed_render.placement_id = Some(20);
    changed_render.image_data = KittyImageData::Png {
        data: vec![8, 8, 8, 8],
        width: 1,
        height: 1,
    };
    let desired = create_kitty_scene_state(vec![
        PlannedKittyPlacement::Explicit {
            key: KittyPlacementKey {
                image_id: 1,
                placement_id: 10,
            },
            chunk: changed_chunk.clone(),
        },
        PlannedKittyPlacement::Placeholder {
            key: KittyPlacementKey {
                image_id: 1,
                placement_id: 20,
            },
            render: changed_render.clone(),
        },
    ]);

    let plan = plan_kitty_scene(&assumed, &desired);

    assert_eq!(
        plan,
        KittyScenePlan::Diff {
            asset_ops: vec![KittyAssetOp::EnsureResident {
                image_id: 1,
                image_data: KittyImageData::Png {
                    data: vec![8, 8, 8, 8],
                    width: 1,
                    height: 1,
                },
            }],
            placement_ops: vec![
                KittyPlacementOp::Delete {
                    key: KittyPlacementKey {
                        image_id: 1,
                        placement_id: 10,
                    },
                },
                KittyPlacementOp::Delete {
                    key: KittyPlacementKey {
                        image_id: 1,
                        placement_id: 20,
                    },
                },
                KittyPlacementOp::PlaceExplicit {
                    key: KittyPlacementKey {
                        image_id: 1,
                        placement_id: 10,
                    },
                    chunk: changed_chunk,
                },
                KittyPlacementOp::PlacePlaceholder {
                    key: KittyPlacementKey {
                        image_id: 1,
                        placement_id: 20,
                    },
                    render: changed_render,
                },
            ],
        }
    );
}

#[test]
fn test_is_dirty_with_kitty_scene_diffs() {
    let client_ids = create_test_clients(1);
    let base_chunk = create_kitty_chunk(1, 2, 2);
    let changed_chunk = create_kitty_chunk(1, 3, 2);
    let kitty_delete_all = "\u{1b}_Ga=d,d=A\u{1b}\\";

    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(vec![base_chunk.clone()]),
        None,
    );
    assert!(output.is_dirty(), "new kitty scene should be dirty");
    let serialized = output.serialize().unwrap();
    assert!(
        !serialized.get(&1).unwrap().contains(kitty_delete_all),
        "new kitty scene should not require delete-all"
    );

    let mut unchanged_output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    unchanged_output.add_clients(&client_ids, link_handler, None);
    unchanged_output.set_last_rendered_kitty_chunks(
        HashMap::from([(1, vec![base_chunk.clone()])]),
        HashMap::new(),
    );
    unchanged_output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(vec![base_chunk.clone()]),
        None,
    );
    let serialized = unchanged_output.serialize().unwrap();
    assert!(
        !serialized.get(&1).unwrap().contains(kitty_delete_all),
        "identical kitty scene should not force a kitty clear"
    );

    let mut changed_output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    changed_output.add_clients(&client_ids, link_handler, None);
    changed_output.set_last_rendered_kitty_chunks(
        HashMap::from([(1, vec![base_chunk.clone()])]),
        HashMap::new(),
    );
    changed_output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(vec![changed_chunk]),
        None,
    );
    assert!(
        changed_output.is_dirty(),
        "changed kitty scene should be dirty"
    );
    let serialized = changed_output.serialize().unwrap();
    assert!(
        !serialized.get(&1).unwrap().contains(kitty_delete_all),
        "changed kitty scene should not force a kitty delete-all"
    );

    let mut cleared_output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    cleared_output.add_clients(&client_ids, link_handler, None);
    cleared_output
        .set_last_rendered_kitty_chunks(HashMap::from([(1, vec![base_chunk])]), HashMap::new());
    assert!(
        cleared_output.is_dirty(),
        "clearing a previously rendered kitty scene should be dirty"
    );
    let serialized = cleared_output.serialize().unwrap();
    let client_output = serialized.get(&1).unwrap();
    assert!(
        !client_output.contains(kitty_delete_all),
        "clearing a previously rendered kitty scene should use targeted deletes"
    );
    assert!(
        client_output.contains("a=d,d=i"),
        "clearing a previously rendered kitty scene should emit placement deletes"
    );
}

#[test]
fn test_serialize_emits_kitty_damage_redraw_without_scene_change() {
    let client_ids = create_test_clients(1);
    let mut base_chunk = create_kitty_chunk(1, 2, 2);
    base_chunk.cell_y = 4;

    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.set_last_rendered_kitty_chunks(
        HashMap::from([(1, vec![base_chunk.clone()])]),
        HashMap::new(),
    );
    output.add_pane_image_output_to_client(
        1,
        PaneImageRenderOutput {
            kitty_scene: crate::panes::pane_image_scene::KittyRenderBundle {
                explicit_chunks: vec![base_chunk],
                placeholder_renders: vec![],
            },
            changed_rects: HashMap::from([(4, 2)]),
            ..Default::default()
        },
        None,
    );

    assert!(
        output.is_dirty(),
        "kitty damage redraws should make output dirty even when the scene is unchanged"
    );

    let serialized = output.serialize().unwrap();
    let client_output = serialized.get(&1).unwrap();
    assert!(
        !client_output.contains("\u{1b}_Ga=d,d=A\u{1b}\\"),
        "kitty damage redraw should not force a full-scene clear when the scene is unchanged"
    );
    assert!(
        client_output.contains("a=p"),
        "kitty damage redraw should still place kitty output when the scene is unchanged"
    );
}

#[test]
fn test_image_output_adds_placement_without_full_scene_reset() {
    let client_ids = create_test_clients(1);
    let mut first_chunk = create_kitty_chunk(1, 2, 2);
    first_chunk.placement_id = Some(10);
    let mut second_chunk = create_kitty_chunk(2, 2, 2);
    second_chunk.placement_id = Some(20);
    second_chunk.cell_x = 5;

    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.set_last_rendered_kitty_chunks(
        HashMap::from([(1, vec![first_chunk.clone()])]),
        HashMap::new(),
    );
    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(vec![first_chunk, second_chunk]),
        None,
    );

    let serialized = output.serialize().unwrap();
    let client_output = serialized.get(&1).unwrap();
    assert!(
        !client_output.contains("\u{1b}_Ga=d,d=A\u{1b}\\"),
        "adding a placement should not require a kitty delete-all"
    );
}

#[test]
fn test_image_output_removes_single_placement_without_delete_all() {
    let client_ids = create_test_clients(1);
    let mut first_chunk = create_kitty_chunk(1, 2, 2);
    first_chunk.placement_id = Some(10);
    let mut second_chunk = create_kitty_chunk(2, 2, 2);
    second_chunk.placement_id = Some(20);
    second_chunk.cell_x = 5;

    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.set_last_rendered_kitty_chunks(
        HashMap::from([(1, vec![first_chunk.clone(), second_chunk])]),
        HashMap::new(),
    );
    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(vec![first_chunk]),
        None,
    );

    let serialized = output.serialize().unwrap();
    let client_output = serialized.get(&1).unwrap();
    assert!(
        !client_output.contains("\u{1b}_Ga=d,d=A\u{1b}\\"),
        "removing one placement should not require a kitty delete-all"
    );
}

#[test]
fn test_image_output_replaces_changed_geometry_without_resetting_unrelated_placements() {
    let client_ids = create_test_clients(1);
    let mut first_chunk = create_kitty_chunk(1, 2, 2);
    first_chunk.placement_id = Some(10);
    let mut second_chunk = create_kitty_chunk(2, 2, 2);
    second_chunk.placement_id = Some(20);
    second_chunk.cell_x = 5;

    let mut changed_second_chunk = second_chunk.clone();
    changed_second_chunk.columns = 3;

    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.set_last_rendered_kitty_chunks(
        HashMap::from([(1, vec![first_chunk.clone(), second_chunk])]),
        HashMap::new(),
    );
    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(vec![first_chunk, changed_second_chunk]),
        None,
    );

    let serialized = output.serialize().unwrap();
    let client_output = serialized.get(&1).unwrap();
    assert!(
        !client_output.contains("\u{1b}_Ga=d,d=A\u{1b}\\"),
        "changing one placement should not reset unrelated placements"
    );
}

#[test]
fn test_image_output_asset_change_invalidates_all_referencing_placements() {
    let client_ids = create_test_clients(1);
    let mut first_chunk = create_kitty_chunk(1, 2, 2);
    first_chunk.placement_id = Some(10);
    let mut second_chunk = create_kitty_chunk(1, 2, 2);
    second_chunk.placement_id = Some(11);
    second_chunk.cell_x = 5;
    let mut other_asset_chunk = create_kitty_chunk(2, 2, 2);
    other_asset_chunk.placement_id = Some(20);
    other_asset_chunk.cell_x = 10;

    let mut changed_first_chunk = first_chunk.clone();
    changed_first_chunk.image_data = KittyImageData::Png {
        data: vec![7, 7, 7, 7],
        width: 1,
        height: 1,
    };
    let mut changed_second_chunk = second_chunk.clone();
    changed_second_chunk.image_data = KittyImageData::Png {
        data: vec![7, 7, 7, 7],
        width: 1,
        height: 1,
    };

    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.set_last_rendered_kitty_chunks(
        HashMap::from([(
            1,
            vec![first_chunk, second_chunk, other_asset_chunk.clone()],
        )]),
        HashMap::new(),
    );
    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(vec![
            changed_first_chunk,
            changed_second_chunk,
            other_asset_chunk,
        ]),
        None,
    );

    let serialized = output.serialize().unwrap();
    let client_output = serialized.get(&1).unwrap();
    assert!(
        !client_output.contains("\u{1b}_Ga=d,d=A\u{1b}\\"),
        "asset changes should not force a kitty delete-all when per-asset updates suffice"
    );
}

#[test]
fn test_image_output_pre_vte_clear_invalidates_assumed_kitty_scene() {
    let client_ids = create_test_clients(1);
    let mut chunk = create_kitty_chunk(1, 2, 2);
    chunk.placement_id = Some(10);

    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output
        .set_last_rendered_kitty_chunks(HashMap::from([(1, vec![chunk.clone()])]), HashMap::new());
    output.add_pre_vte_instruction_to_client(1, "\u{1b}[2J");
    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(vec![chunk]),
        None,
    );

    let serialized = output.serialize().unwrap();
    let client_output = serialized.get(&1).unwrap();
    assert!(
        !client_output.contains("\u{1b}_Ga=d,d=A\u{1b}\\"),
        "a pre-VTE display clear should invalidate assumed kitty state without emitting a second kitty delete-all"
    );
    assert!(
        client_output.contains("a=p"),
        "after a pre-VTE clear the kitty scene should still be rebuilt"
    );
}

#[test]
fn test_kitty_diff_serialization_deletes_single_placement_without_delete_all() {
    let client_ids = create_test_clients(1);
    let mut first_chunk = create_kitty_chunk(1, 2, 2);
    first_chunk.placement_id = Some(10);
    let mut second_chunk = create_kitty_chunk(2, 2, 2);
    second_chunk.placement_id = Some(20);

    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.set_last_rendered_kitty_chunks(
        HashMap::from([(1, vec![first_chunk.clone(), second_chunk])]),
        HashMap::new(),
    );
    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(vec![first_chunk]),
        None,
    );

    let serialized = output.serialize().unwrap();
    let client_output = serialized.get(&1).unwrap();
    assert!(
        !client_output.contains("\u{1b}_Ga=d,d=A\u{1b}\\"),
        "single-placement delete should not use kitty delete-all"
    );
    assert!(
        client_output.contains("\u{1b}_Ga=d,d=i,i=2,p=20\u{1b}\\"),
        "single-placement delete should target only the removed placement"
    );
}

#[test]
fn test_kitty_diff_serialization_places_resident_asset_without_retransmit() {
    let client_ids = create_test_clients(1);
    let mut chunk = create_kitty_chunk(1, 2, 2);
    chunk.placement_id = Some(10);

    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output
        .set_last_rendered_kitty_chunks(HashMap::from([(1, vec![chunk.clone()])]), HashMap::new());
    output.add_pane_image_output_to_client(1, pane_image_output_with_kitty_scene(vec![]), None);
    output.serialize().unwrap();

    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(vec![chunk]),
        None,
    );
    let serialized = output.serialize().unwrap();
    let client_output = serialized.get(&1).unwrap();

    assert!(
        !client_output.contains("a=t"),
        "re-placing a resident asset should not retransmit image bytes"
    );
    assert!(
        client_output.contains("a=p"),
        "re-placing a resident asset should still emit a kitty place op"
    );
}

#[test]
fn test_kitty_diff_serialization_transmits_asset_before_new_placement() {
    let client_ids = create_test_clients(1);
    let mut chunk = create_kitty_chunk(1, 2, 2);
    chunk.placement_id = Some(10);

    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(vec![chunk]),
        None,
    );

    let serialized = output.serialize().unwrap();
    let client_output = serialized.get(&1).unwrap();
    let transmit_pos = client_output.find("a=t").unwrap();
    let place_pos = client_output.find("a=p").unwrap();

    assert!(
        transmit_pos < place_pos,
        "missing assets should be transmitted before their placement commands"
    );
}

#[test]
fn test_kitty_diff_serialization_full_reset_fallback_preserves_existing_behavior() {
    let client_ids = create_test_clients(1);
    let mut base_chunk = create_kitty_chunk(1, 2, 2);
    base_chunk.placement_id = None;
    let mut changed_chunk = base_chunk.clone();
    changed_chunk.columns = 3;

    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.set_last_rendered_kitty_chunks(HashMap::from([(1, vec![base_chunk])]), HashMap::new());
    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(vec![changed_chunk]),
        None,
    );

    let serialized = output.serialize().unwrap();
    let client_output = serialized.get(&1).unwrap();

    assert!(
        client_output.contains("\u{1b}_Ga=d,d=A\u{1b}\\"),
        "unsupported kitty diff cases should still fall back to delete-all"
    );
    assert!(
        client_output.contains("a=t") && client_output.contains("a=p"),
        "full-reset fallback should still rebuild the scene"
    );
}

#[test]
fn test_prepared_image_output_emits_kitty_delete_before_text_when_scene_changes() {
    let client_ids = create_test_clients(1);
    let mut base_chunk = create_kitty_chunk(77, 2, 2);
    base_chunk.placement_id = Some(10);
    let mut changed_chunk = base_chunk.clone();
    changed_chunk.columns = 3;
    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.set_last_rendered_kitty_chunks(HashMap::from([(1, vec![base_chunk])]), HashMap::new());

    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(vec![changed_chunk]),
        None,
    );
    let chunk = create_character_chunk_from_str("TEXT-PHASE", 0, 0);
    output
        .add_character_chunks_to_client(1, vec![chunk], None)
        .unwrap();

    let serialized = output.serialize().unwrap();
    let client_output = serialized.get(&1).unwrap();
    let delete_pos = client_output.find("a=d,d=i,i=77,p=10").unwrap();
    let text_pos = client_output.find("TEXT-PHASE").unwrap();

    assert!(
        delete_pos < text_pos,
        "kitty placement deletes should happen before text when the scene changes"
    );
}

#[test]
fn test_prepared_image_output_serializes_sixels_after_text() {
    let client_ids = create_test_clients(1);
    let (mut output, sixel_image_store, character_cell_size) = create_test_output_with_state();
    seed_test_sixel_image(sixel_image_store, character_cell_size, 1);
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);

    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_sixels(vec![SixelImageChunk {
            cell_x: 0,
            cell_y: 0,
            sixel_image_pixel_x: 0,
            sixel_image_pixel_y: 0,
            sixel_image_pixel_width: 14,
            sixel_image_pixel_height: 12,
            sixel_image_id: 1,
        }]),
        None,
    );
    let chunk = create_character_chunk_from_str("TEXT-BEFORE-SIXEL", 0, 1);
    output
        .add_character_chunks_to_client(1, vec![chunk], None)
        .unwrap();

    let serialized = output.serialize().unwrap();
    let client_output = serialized.get(&1).unwrap();
    let text_pos = client_output.find("TEXT-BEFORE-SIXEL").unwrap();
    let sixel_pos = client_output.find("\u{1b}P").unwrap();

    assert!(
        text_pos < sixel_pos,
        "sixel output should be appended after text serialization"
    );
}

#[test]
fn test_prepared_image_output_serializes_kitty_after_text() {
    let client_ids = create_test_clients(1);
    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);

    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_placeholder(vec![create_kitty_placeholder_render(88)]),
        None,
    );
    let chunk = create_character_chunk_from_str("TEXT-BEFORE-KITTY", 0, 1);
    output
        .add_character_chunks_to_client(1, vec![chunk], None)
        .unwrap();

    let serialized = output.serialize().unwrap();
    let client_output = serialized.get(&1).unwrap();
    let text_pos = client_output.find("TEXT-BEFORE-KITTY").unwrap();
    let kitty_pos = client_output.rfind("\u{1b}_G").unwrap();

    assert!(
        text_pos < kitty_pos,
        "kitty image serialization should happen after text serialization"
    );
}

#[test]
fn test_prepare_render_body_derives_sixel_fragments() {
    let client_ids = create_test_clients(1);
    let (mut output, sixel_image_store, character_cell_size) = create_test_output_with_state();
    seed_test_sixel_image(sixel_image_store, character_cell_size, 1);
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);

    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_sixels(vec![SixelImageChunk {
            cell_x: 0,
            cell_y: 0,
            sixel_image_pixel_x: 0,
            sixel_image_pixel_y: 0,
            sixel_image_pixel_width: 14,
            sixel_image_pixel_height: 12,
            sixel_image_id: 1,
        }]),
        None,
    );

    let prepared = output.image_output.prepare_render_body_for_client(1, false);
    assert_eq!(prepared.after_text.fragments.len(), 1);
    match &prepared.after_text.fragments[0] {
        ImageFragment::Sixel(fragment) => {
            assert_eq!(fragment.chunk.sixel_image_id, 1);
        },
        other => panic!("expected sixel fragment, got {other:?}"),
    }
}

#[test]
fn test_prepare_render_body_derives_kitty_explicit_fragments() {
    let client_ids = create_test_clients(1);
    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);

    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(vec![create_kitty_chunk(91, 2, 2)]),
        None,
    );

    let prepared = output.image_output.prepare_render_body_for_client(1, false);
    assert!(prepared.before_text_vte.is_none());
    assert!(prepared.after_text.fragments.is_empty());
    assert_eq!(
        prepared.after_text.kitty_plan,
        KittyScenePlan::Diff {
            asset_ops: vec![KittyAssetOp::EnsureResident {
                image_id: 91,
                image_data: KittyImageData::Png {
                    data: vec![1, 2, 3, 4],
                    width: 1,
                    height: 1,
                },
            }],
            placement_ops: vec![KittyPlacementOp::PlaceExplicit {
                key: KittyPlacementKey {
                    image_id: 91,
                    placement_id: 91,
                },
                chunk: create_kitty_chunk(91, 2, 2),
            }],
        }
    );
}

#[test]
fn test_prepare_render_body_derives_kitty_placeholder_fragments() {
    let client_ids = create_test_clients(1);
    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);

    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_placeholder(vec![create_kitty_placeholder_render(92)]),
        None,
    );

    let prepared = output.image_output.prepare_render_body_for_client(1, false);
    assert!(prepared.before_text_vte.is_none());
    assert!(prepared.after_text.fragments.is_empty());
    assert_eq!(
        prepared.after_text.kitty_plan,
        KittyScenePlan::Diff {
            asset_ops: vec![KittyAssetOp::EnsureResident {
                image_id: 92,
                image_data: KittyImageData::Png {
                    data: vec![1, 2, 3, 4],
                    width: 1,
                    height: 1,
                },
            }],
            placement_ops: vec![KittyPlacementOp::PlacePlaceholder {
                key: KittyPlacementKey {
                    image_id: 92,
                    placement_id: 92,
                },
                render: create_kitty_placeholder_render(92),
            }],
        }
    );
}

#[test]
fn test_clip_kitty_explicit_fragment_against_covering_pane() {
    let stack = FloatingPanesStack {
        layers: vec![create_pane_geom(1, 0, 1, 2)],
    };
    let chunk = KittyImageChunk {
        cell_x: 0,
        cell_y: 0,
        columns: 2,
        rows: 2,
        ..create_kitty_chunk(93, 2, 2)
    };
    let fragments = visible_image_fragments(
        &stack,
        vec![ImageFragment::KittyExplicit(KittyExplicitFragment {
            chunk,
        })],
        Some(0),
        None,
    );

    assert_eq!(fragments.len(), 1);
    match &fragments[0] {
        ImageFragment::KittyExplicit(fragment) => {
            assert_eq!(fragment.chunk.cell_x, 0);
            assert_eq!(fragment.chunk.columns, 1);
        },
        other => panic!("expected kitty explicit fragment, got {other:?}"),
    }
}

#[test]
fn test_clip_sixel_fragment_against_covering_pane() {
    let stack = FloatingPanesStack {
        layers: vec![create_pane_geom(0, 0, 3, 3)],
    };
    let fragments = visible_image_fragments(
        &stack,
        vec![ImageFragment::Sixel(SixelFragment {
            chunk: SixelImageChunk {
                cell_x: 0,
                cell_y: 0,
                sixel_image_pixel_x: 0,
                sixel_image_pixel_y: 0,
                sixel_image_pixel_width: 20,
                sixel_image_pixel_height: 40,
                sixel_image_id: 1,
            },
        })],
        Some(0),
        Some(&SizeInPixels {
            width: 10,
            height: 20,
        }),
    );

    assert!(
        fragments.is_empty(),
        "fully covered sixel fragments should be removed"
    );
}

#[test]
fn test_clip_kitty_placeholder_fragment_against_covering_pane() {
    let stack = FloatingPanesStack {
        layers: vec![create_pane_geom(0, 0, 1, 1)],
    };
    let mut render = create_kitty_placeholder_render(94);
    render.cells = vec![
        crate::output::KittyPlaceholderCellRender {
            cell_x: 0,
            cell_y: 0,
            placeholder_row: 0,
            placeholder_col: 0,
        },
        crate::output::KittyPlaceholderCellRender {
            cell_x: 1,
            cell_y: 0,
            placeholder_row: 0,
            placeholder_col: 1,
        },
    ];
    let fragments = visible_image_fragments(
        &stack,
        vec![ImageFragment::KittyPlaceholder(KittyPlaceholderFragment {
            render,
        })],
        Some(0),
        None,
    );

    assert_eq!(fragments.len(), 1);
    match &fragments[0] {
        ImageFragment::KittyPlaceholder(fragment) => {
            assert_eq!(fragment.render.cells.len(), 1);
            assert_eq!(fragment.render.cells[0].cell_x, 1);
        },
        other => panic!("expected kitty placeholder fragment, got {other:?}"),
    }
}

#[test]
fn test_has_rendered_assets_empty() {
    let output = create_test_output();
    assert!(
        !output.has_rendered_assets(),
        "Empty output should not have rendered assets"
    );
}

#[test]
fn test_has_rendered_assets_only_vte_instructions() {
    let mut output = create_test_output();
    let client_ids = create_test_clients(1);
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);

    output.add_pre_vte_instruction_to_client(1, "\u{1b}[?25l");
    output.add_post_vte_instruction_to_client(1, "\u{1b}[?25h");

    assert!(
        !output.has_rendered_assets(),
        "VTE instructions alone should not count as rendered assets"
    );
}

#[test]
fn test_has_rendered_assets_with_character_chunks() {
    let mut output = create_test_output();
    let client_ids = create_test_clients(1);
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);

    let chunk = create_character_chunk_from_str("Hello", 0, 0);
    output
        .add_character_chunks_to_client(1, vec![chunk], None)
        .unwrap();

    assert!(
        output.has_rendered_assets(),
        "Character chunks should count as rendered assets"
    );
}

#[test]
fn test_has_rendered_assets_with_sixel_chunks() {
    let mut output = create_test_output();
    let client_ids = create_test_clients(1);
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);

    let sixel_chunk = SixelImageChunk {
        cell_x: 0,
        cell_y: 0,
        sixel_image_pixel_x: 0,
        sixel_image_pixel_y: 0,
        sixel_image_pixel_width: 100,
        sixel_image_pixel_height: 100,
        sixel_image_id: 1,
    };
    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_sixels(vec![sixel_chunk]),
        None,
    );

    assert!(
        output.has_rendered_assets(),
        "Sixel chunks should count as rendered assets"
    );
}

#[test]
fn test_serialize_empty() {
    let mut output = create_test_output();
    let result = output.serialize().unwrap();
    assert!(
        result.is_empty(),
        "Serializing empty output should return empty HashMap"
    );
}

#[test]
fn test_serialize_single_client_simple_text() {
    let mut output = create_test_output();
    let client_ids = create_test_clients(1);
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);

    let chunk = create_character_chunk_from_str("Hello", 5, 10);
    output
        .add_character_chunks_to_client(1, vec![chunk], None)
        .unwrap();

    let result = output.serialize().unwrap();
    assert_eq!(result.len(), 1, "Should have one client in result");

    let client_output = result.get(&1).unwrap();
    // Verify contains goto instruction (y+1, x+1 for 1-indexed VTE)
    assert!(
        client_output.contains("\u{1b}[11;6H"),
        "Should contain goto instruction for position (5, 10)"
    );
    // Verify contains reset styles
    assert!(
        client_output.contains("\u{1b}[m"),
        "Should contain reset styles"
    );
    // Verify contains the text
    assert!(client_output.contains("Hello"), "Should contain the text");
}

#[test]
fn test_serialize_multiple_clients() {
    let mut output = create_test_output();
    let client_ids = create_test_clients(2);
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);

    let chunk1 = create_character_chunk_from_str("Hello", 0, 0);
    output
        .add_character_chunks_to_client(1, vec![chunk1], None)
        .unwrap();

    let chunk2 = create_character_chunk_from_str("World", 10, 10);
    output
        .add_character_chunks_to_client(2, vec![chunk2], None)
        .unwrap();

    let result = output.serialize().unwrap();
    assert_eq!(result.len(), 2, "Should have two clients in result");

    let client1_output = result.get(&1).unwrap();
    assert!(
        client1_output.contains("Hello"),
        "Client 1 should contain 'Hello'"
    );

    let client2_output = result.get(&2).unwrap();
    assert!(
        client2_output.contains("World"),
        "Client 2 should contain 'World'"
    );
}

#[test]
fn test_serialize_with_pre_and_post_vte_instructions() {
    let mut output = create_test_output();
    let client_ids = create_test_clients(1);
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);

    output.add_pre_vte_instruction_to_client(1, "\u{1b}[?1049h");
    let chunk = create_character_chunk_from_str("Test", 0, 0);
    output
        .add_character_chunks_to_client(1, vec![chunk], None)
        .unwrap();
    output.add_post_vte_instruction_to_client(1, "\u{1b}[?25h");

    let result = output.serialize().unwrap();
    let client_output = result.get(&1).unwrap();

    // Verify correct ordering
    let pre_vte_pos = client_output.find("\u{1b}[?1049h").unwrap();
    let text_pos = client_output.find("Test").unwrap();
    let post_vte_pos = client_output.find("\u{1b}[?25h").unwrap();

    assert!(
        pre_vte_pos < text_pos && text_pos < post_vte_pos,
        "Instructions should be in correct order: pre, content, post"
    );
}

#[test]
fn test_serialize_drains_state() {
    let mut output = create_test_output();
    let client_ids = create_test_clients(1);
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);

    let chunk = create_character_chunk_from_str("Hello", 0, 0);
    output
        .add_character_chunks_to_client(1, vec![chunk], None)
        .unwrap();

    assert!(output.is_dirty(), "Output should be dirty before serialize");

    let result1 = output.serialize().unwrap();
    assert_eq!(result1.len(), 1, "First serialize should return data");

    assert!(
        !output.is_dirty(),
        "Output should not be dirty after serialize"
    );

    let result2 = output.serialize().unwrap();
    assert!(
        result2.is_empty(),
        "Second serialize should return empty HashMap"
    );
}

#[test]
fn test_serialize_with_size_no_constraints() {
    let mut output = create_test_output();
    let client_ids = create_test_clients(1);
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);

    let chunk = create_character_chunk_from_str("Hello", 0, 0);
    output
        .add_character_chunks_to_client(1, vec![chunk], None)
        .unwrap();

    let result = output.serialize_with_size(None, None).unwrap();
    assert_eq!(result.len(), 1, "Should have one client in result");

    let client_output = result.get(&1).unwrap();
    assert!(client_output.contains("Hello"), "Should contain the text");
}

#[test]
fn test_serialize_with_size_crops_chunks_below_visible_area() {
    let mut output = create_test_output();
    let client_ids = create_test_clients(1);
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);

    let max_size = Some(Size { rows: 10, cols: 80 });

    // Add chunk below visible area (should be cropped)
    let chunk_below = create_character_chunk_from_str("Hidden", 0, 15);
    output
        .add_character_chunks_to_client(1, vec![chunk_below], None)
        .unwrap();

    // Add chunk within visible area (should be included)
    let chunk_visible = create_character_chunk_from_str("Visible", 0, 5);
    output
        .add_character_chunks_to_client(1, vec![chunk_visible], None)
        .unwrap();

    let result = output.serialize_with_size(max_size, None).unwrap();
    let client_output = result.get(&1).unwrap();

    assert!(
        client_output.contains("Visible"),
        "Should contain visible chunk"
    );
    assert!(
        !client_output.contains("Hidden"),
        "Should not contain chunk below visible area"
    );
}

#[test]
fn test_serialize_with_size_crops_chunks_outside_cols() {
    let mut output = create_test_output();
    let client_ids = create_test_clients(1);
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);

    let max_size = Some(Size { rows: 10, cols: 20 });

    // Add chunk outside visible columns (should be cropped)
    let chunk_outside = create_character_chunk_from_str("Hidden", 25, 5);
    output
        .add_character_chunks_to_client(1, vec![chunk_outside], None)
        .unwrap();

    // Add chunk within visible area
    let chunk_visible = create_character_chunk_from_str("Visible", 5, 5);
    output
        .add_character_chunks_to_client(1, vec![chunk_visible], None)
        .unwrap();

    let result = output.serialize_with_size(max_size, None).unwrap();
    let client_output = result.get(&1).unwrap();

    assert!(
        client_output.contains("Visible"),
        "Should contain visible chunk"
    );
    assert!(
        !client_output.contains("Hidden"),
        "Should not contain chunk outside visible columns"
    );
}

#[test]
fn test_serialize_with_size_crops_characters_within_chunk() {
    let mut output = create_test_output();
    let client_ids = create_test_clients(1);
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);

    let max_size = Some(Size { rows: 10, cols: 20 });

    // Add chunk that starts at x=15 and would extend to x=25 (10 chars)
    let chunk = create_character_chunk_from_str("1234567890", 15, 5);
    output
        .add_character_chunks_to_client(1, vec![chunk], None)
        .unwrap();

    let result = output.serialize_with_size(max_size, None).unwrap();
    let client_output = result.get(&1).unwrap();

    // Should only render first 5 characters (cols 15-19)
    assert!(
        client_output.contains("12345"),
        "Should contain first 5 characters"
    );
    assert!(
        !client_output.contains("67890"),
        "Should not contain characters beyond max_size.cols"
    );
}

#[test]
fn test_serialize_with_size_adds_padding_instructions() {
    let mut output = create_test_output();
    let client_ids = create_test_clients(1);
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);

    let max_size = Some(Size {
        rows: 30,
        cols: 100,
    });
    let content_size = Some(Size { rows: 20, cols: 80 });

    let chunk = create_character_chunk_from_str("Test", 0, 0);
    output
        .add_character_chunks_to_client(1, vec![chunk], None)
        .unwrap();

    let result = output.serialize_with_size(max_size, content_size).unwrap();
    let client_output = result.get(&1).unwrap();

    // Verify padding/clearing instructions are present
    // Should contain clear line instructions: \u{1b}[y;xH\u{1b}[m\u{1b}[K
    assert!(
        client_output.contains("\u{1b}[K"),
        "Should contain clear line instructions"
    );
    // Should contain clear below instruction: \u{1b}[21;1H\u{1b}[m\u{1b}[J
    assert!(
        client_output.contains("\u{1b}[21;1H\u{1b}[m\u{1b}[J"),
        "Should contain clear below instruction at line 21"
    );
}

#[test]
fn test_serialize_with_size_hides_cursor_when_cropped() {
    let mut output = create_test_output();
    let client_ids = create_test_clients(1);
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);

    let max_size = Some(Size { rows: 10, cols: 20 });

    // Set cursor outside max_size
    output.cursor_is_visible(25, 5, None);

    let chunk = create_character_chunk_from_str("Test", 0, 0);
    output
        .add_character_chunks_to_client(1, vec![chunk], None)
        .unwrap();

    let result = output.serialize_with_size(max_size, None).unwrap();
    let client_output = result.get(&1).unwrap();

    // Verify hide cursor instruction is added
    assert!(
        client_output.contains("\u{1b}[?25l"),
        "Should contain hide cursor instruction when cursor is cropped"
    );
}

#[test]
fn test_add_character_chunks_to_multiple_clients() {
    let mut output = create_test_output();
    let client_ids = create_test_clients(3);
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);

    let chunk = create_character_chunk_from_str("Test", 0, 0);
    output
        .add_character_chunks_to_multiple_clients(vec![chunk], client_ids.iter().copied(), None)
        .unwrap();

    let result = output.serialize().unwrap();
    assert_eq!(result.len(), 3, "Should have three clients in result");

    for client_id in 1..=3 {
        let client_output = result.get(&client_id).unwrap();
        assert!(
            client_output.contains("Test"),
            "Client {} should contain the text",
            client_id
        );
    }
}

#[test]
fn test_add_sixel_image_chunks_to_multiple_clients() {
    let mut output = create_test_output();
    let client_ids = create_test_clients(2);
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);

    let sixel_chunk = SixelImageChunk {
        cell_x: 0,
        cell_y: 0,
        sixel_image_pixel_x: 0,
        sixel_image_pixel_y: 0,
        sixel_image_pixel_width: 100,
        sixel_image_pixel_height: 100,
        sixel_image_id: 1,
    };

    output.add_pane_image_output_to_multiple_clients(
        pane_image_output_with_sixels(vec![sixel_chunk]),
        client_ids.iter().copied(),
        None,
    );

    assert!(
        output.has_rendered_assets(),
        "Output should have rendered assets"
    );
}

#[test]
fn test_character_chunk_new() {
    let terminal_chars: Vec<TerminalCharacter> = vec![TerminalCharacter::new('A')];

    let chunk = CharacterChunk::new(terminal_chars, 5, 10);

    assert_eq!(chunk.x, 5, "x should be set correctly");
    assert_eq!(chunk.y, 10, "y should be set correctly");
    assert_eq!(
        chunk.terminal_characters.len(),
        1,
        "Should have one character"
    );
}

#[test]
fn test_character_chunk_width() {
    let chunk = create_character_chunk_from_str("Hello", 0, 0);
    assert_eq!(chunk.width(), 5, "Width should be 5 for 'Hello'");

    // Test with wide characters
    let terminal_chars: Vec<TerminalCharacter> = vec![
        TerminalCharacter::new('a'),
        TerminalCharacter::new('中'),
        TerminalCharacter::new('b'),
    ];
    let chunk_wide = CharacterChunk::new(terminal_chars, 0, 0);
    assert_eq!(
        chunk_wide.width(),
        4,
        "Width should be 4 (1 + 2 + 1) for mixed characters"
    );
}

#[test]
fn test_character_chunk_drain_by_width() {
    let mut chunk = create_character_chunk_from_str("Hello World", 0, 0);
    assert_eq!(chunk.width(), 11, "Initial width should be 11");

    // Drain first 5 characters
    let drained: Vec<TerminalCharacter> = chunk.drain_by_width(5).collect();
    assert_eq!(drained.len(), 5, "Should drain 5 characters");
    assert_eq!(
        chunk.terminal_characters.len(),
        6,
        "Should have 6 characters remaining"
    );

    let drained_text: String = drained.iter().map(|c| c.character).collect();
    assert_eq!(drained_text, "Hello", "Drained part should be 'Hello'");

    let remaining_text: String = chunk
        .terminal_characters
        .iter()
        .map(|c| c.character)
        .collect();
    assert_eq!(
        remaining_text, " World",
        "Remaining part should be ' World'"
    );
}

#[test]
fn test_character_chunk_drain_by_width_with_wide_chars() {
    let terminal_chars: Vec<TerminalCharacter> = vec![
        TerminalCharacter::new('a'),
        TerminalCharacter::new('中'),
        TerminalCharacter::new('b'),
    ];
    let mut chunk = CharacterChunk::new(terminal_chars, 0, 0);

    // Drain 2 characters - this cuts in the middle of wide char
    let drained: Vec<TerminalCharacter> = chunk.drain_by_width(2).collect();

    // Should have padding with EMPTY_TERMINAL_CHARACTER
    assert!(
        drained.len() >= 2,
        "Drained part should have at least 2 characters"
    );
}

#[test]
fn test_character_chunk_retain_by_width() {
    let mut chunk = create_character_chunk_from_str("Hello World", 0, 0);

    // Retain only first 5 characters
    chunk.retain_by_width(5);

    assert_eq!(
        chunk.terminal_characters.len(),
        5,
        "Should have 5 characters"
    );
    let text: String = chunk
        .terminal_characters
        .iter()
        .map(|c| c.character)
        .collect();
    assert_eq!(text, "Hello", "Should retain 'Hello'");
}

#[test]
fn test_character_chunk_cut_middle_out() {
    let mut chunk = create_character_chunk_from_str("Hello World", 0, 0);

    // Cut middle (characters 5-8)
    let (left, right) = chunk.cut_middle_out(5, 8).unwrap();

    let left_text: String = left.iter().map(|c| c.character).collect();
    assert_eq!(left_text, "Hello", "Left chunk should be 'Hello'");

    let right_text: String = right.iter().map(|c| c.character).collect();
    assert_eq!(right_text, "rld", "Right chunk should be 'rld'");
}

#[test]
fn test_visible_character_chunks_no_panes() {
    let stack = FloatingPanesStack { layers: vec![] };
    let chunks = vec![create_character_chunk_from_str("Test", 0, 0)];

    let visible = stack.visible_character_chunks(chunks, None).unwrap();

    assert_eq!(visible.len(), 1, "All chunks should be visible");
}

#[test]
fn test_visible_character_chunks_completely_covered() {
    let pane_geom = create_pane_geom(0, 0, 10, 10);
    let stack = FloatingPanesStack {
        layers: vec![pane_geom],
    };

    // Chunk completely within pane bounds
    let chunks = vec![create_character_chunk_from_str("Test", 5, 5)];

    let visible = stack.visible_character_chunks(chunks, Some(0)).unwrap();

    assert_eq!(
        visible.len(),
        0,
        "Completely covered chunk should not be visible"
    );
}

#[test]
fn test_visible_character_chunks_partially_covered_left() {
    let pane_geom = create_pane_geom(0, 5, 10, 1);
    let stack = FloatingPanesStack {
        layers: vec![pane_geom],
    };

    // Chunk that spans x=5-15, pane covers x=0-9
    let chunks = vec![create_character_chunk_from_str("HelloWorld", 5, 5)];

    let visible = stack.visible_character_chunks(chunks, Some(0)).unwrap();

    // Should retain the right part
    assert!(visible.len() > 0, "Should have visible chunks");
    if !visible.is_empty() {
        assert!(
            visible[0].x >= 10,
            "Visible chunk should start after pane edge"
        );
    }
}

#[test]
fn test_visible_character_chunks_partially_covered_right() {
    let pane_geom = create_pane_geom(10, 5, 10, 1);
    let stack = FloatingPanesStack {
        layers: vec![pane_geom],
    };

    // Chunk that spans x=5-15, pane covers x=10-19
    let chunks = vec![create_character_chunk_from_str("HelloWorld", 5, 5)];

    let visible = stack.visible_character_chunks(chunks, Some(0)).unwrap();

    // Should retain the left part
    assert!(visible.len() > 0, "Should have visible chunks");
    if !visible.is_empty() {
        assert_eq!(visible[0].x, 5, "Visible chunk should start at original x");
        assert!(
            visible[0].width() < 10,
            "Visible chunk should be shorter than original"
        );
    }
}

#[test]
fn test_visible_character_chunks_middle_covered() {
    let pane_geom = create_pane_geom(5, 5, 3, 1);
    let stack = FloatingPanesStack {
        layers: vec![pane_geom],
    };

    // Chunk spans x=0-10, pane covers x=5-7
    let chunks = vec![create_character_chunk_from_str("0123456789", 0, 5)];

    let visible = stack.visible_character_chunks(chunks, Some(0)).unwrap();

    // Should return two chunks (left and right parts)
    assert!(
        visible.len() >= 1,
        "Should have at least one visible chunk when middle is covered"
    );
}

#[test]
fn test_cursor_is_visible_with_floating_panes() {
    let pane_geom = create_pane_geom(5, 5, 10, 10);
    let stack = FloatingPanesStack {
        layers: vec![pane_geom],
    };

    // Cursor inside pane bounds
    assert!(
        !stack.cursor_is_visible(7, 7, None),
        "Cursor should not be visible when covered by pane"
    );

    // Cursor outside pane bounds
    assert!(
        stack.cursor_is_visible(20, 20, None),
        "Cursor should be visible when not covered by pane"
    );
}

#[test]
fn test_cursor_visibility_with_z_index_bottom_layer() {
    // Two overlapping panes: bottom at (5,5) and top at (5,5)
    let bottom_pane = create_pane_geom(5, 5, 10, 10);
    let top_pane = create_pane_geom(5, 5, 10, 10);

    let stack = FloatingPanesStack {
        layers: vec![bottom_pane, top_pane],
    };

    // Cursor at position (7,7) in the bottom layer (z-index 0)
    // Should be hidden because there's a pane above it (z-index 1) at the same position
    assert!(
        !stack.cursor_is_visible(7, 7, Some(0)),
        "Cursor in bottom layer should be hidden by pane above it"
    );
}

#[test]
fn test_cursor_visibility_with_z_index_top_layer() {
    // Two overlapping panes: bottom at (5,5) and top at (5,5)
    let bottom_pane = create_pane_geom(5, 5, 10, 10);
    let top_pane = create_pane_geom(5, 5, 10, 10);

    let stack = FloatingPanesStack {
        layers: vec![bottom_pane, top_pane],
    };

    // Cursor at position (7,7) in the top layer (z-index 1)
    // Should be visible even though there's a pane below it at the same position
    assert!(
        stack.cursor_is_visible(7, 7, Some(1)),
        "Cursor in top layer should be visible (not affected by panes below)"
    );
}

#[test]
fn test_cursor_visibility_with_multiple_layers() {
    // Three panes stacked vertically with different layers
    // Layer 0 (bottom): pane at (5,5)
    // Layer 1 (middle): pane at (10,10)
    // Layer 2 (top): pane at (5,5) - overlaps with layer 0
    let layer0_pane = create_pane_geom(5, 5, 10, 10);
    let layer1_pane = create_pane_geom(10, 10, 10, 10);
    let layer2_pane = create_pane_geom(5, 5, 10, 10);

    let stack = FloatingPanesStack {
        layers: vec![layer0_pane, layer1_pane, layer2_pane],
    };

    // Cursor at (7,7) in layer 0 - should be hidden by layer 2
    assert!(
        !stack.cursor_is_visible(7, 7, Some(0)),
        "Cursor in layer 0 should be hidden by layer 2 above it"
    );

    // Cursor at (16,16) in layer 1 - should be visible (layer 2 doesn't cover this position)
    // Layer 1 is at (10,10) size 10x10, so covers (10,10) to (19,19)
    // Layer 2 is at (5,5) size 10x10, so covers (5,5) to (14,14)
    // Position (16,16) is inside layer 1 but outside layer 2
    assert!(
        stack.cursor_is_visible(16, 16, Some(1)),
        "Cursor in layer 1 should be visible when not covered by layers above"
    );

    // Cursor at (7,7) in layer 2 - should be visible (no layers above)
    assert!(
        stack.cursor_is_visible(7, 7, Some(2)),
        "Cursor in top layer should always be visible"
    );
}

#[test]
fn test_cursor_visibility_pinned_pane_over_floating() {
    // Simulates a floating pane (z-index 0) with a pinned pane (z-index 1) on top
    // Both panes overlap at the same position
    let floating_pane = create_pane_geom(10, 10, 20, 20);
    let mut pinned_pane = create_pane_geom(10, 10, 20, 20);
    pinned_pane.is_pinned = true;

    let stack = FloatingPanesStack {
        layers: vec![floating_pane, pinned_pane],
    };

    // Cursor in the floating pane (z-index 0) at position covered by pinned pane
    assert!(
        !stack.cursor_is_visible(15, 15, Some(0)),
        "Cursor in floating pane should be hidden when covered by pinned pane above"
    );

    // Cursor in the pinned pane (z-index 1) at the same position
    assert!(
        stack.cursor_is_visible(15, 15, Some(1)),
        "Cursor in pinned pane should be visible"
    );
}

#[test]
fn test_cursor_visibility_partial_overlap() {
    // Two panes with partial overlap
    // Layer 0: pane at (0, 0) size 10x10
    // Layer 1: pane at (5, 5) size 10x10 (overlaps bottom-right of layer 0)
    let layer0_pane = create_pane_geom(0, 0, 10, 10);
    let layer1_pane = create_pane_geom(5, 5, 10, 10);

    let stack = FloatingPanesStack {
        layers: vec![layer0_pane, layer1_pane],
    };

    // Cursor at (2,2) in layer 0 - not covered by layer 1, should be visible
    assert!(
        stack.cursor_is_visible(2, 2, Some(0)),
        "Cursor in non-overlapping area should be visible"
    );

    // Cursor at (7,7) in layer 0 - covered by layer 1, should be hidden
    assert!(
        !stack.cursor_is_visible(7, 7, Some(0)),
        "Cursor in overlapping area should be hidden by pane above"
    );

    // Cursor at (7,7) in layer 1 - should be visible (top layer)
    assert!(
        stack.cursor_is_visible(7, 7, Some(1)),
        "Cursor in top layer should be visible in overlapping area"
    );
}

#[test]
fn test_output_buffer_update_line() {
    let mut buffer = OutputBuffer::default();
    buffer.clear(); // Clear the initial "update all lines" state

    buffer.update_line(5);

    assert!(
        buffer.changed_lines.contains(&5),
        "Changed lines should contain line 5"
    );
}

#[test]
fn test_output_buffer_update_all_lines() {
    let mut buffer = OutputBuffer::default();

    assert!(
        buffer.should_update_all_lines,
        "Should update all lines by default"
    );

    buffer.clear();
    assert!(
        !buffer.should_update_all_lines,
        "Should not update all lines after clear"
    );

    buffer.update_all_lines();
    assert!(
        buffer.should_update_all_lines,
        "Should update all lines after update_all_lines"
    );
}

#[test]
fn test_output_buffer_serialize() {
    let buffer = OutputBuffer::default();

    // Create a simple viewport with Row data
    let mut columns = VecDeque::new();
    columns.push_back(TerminalCharacter::new('A'));
    let row = Row::from_columns(columns);
    let viewport = vec![row];

    let result = buffer.serialize(&viewport, true, None).unwrap();

    // Should contain the character and newlines/carriage returns
    assert!(result.contains('A'), "Serialized output should contain 'A'");
    assert!(
        result.contains("\n\r"),
        "Serialized output should contain newlines"
    );
}

#[test]
fn test_output_buffer_changed_chunks_in_viewport_when_all_dirty() {
    let buffer = OutputBuffer::default();

    let mut columns = VecDeque::new();
    columns.push_back(TerminalCharacter::new('A'));
    let row = Row::from_columns(columns);
    let viewport = vec![row];

    let chunks = buffer.changed_chunks_in_viewport(&viewport, 10, 1, 0, 0);

    assert_eq!(
        chunks.len(),
        1,
        "Should return all lines when should_update_all_lines is true"
    );
}

#[test]
fn test_output_buffer_changed_chunks_in_viewport_partial() {
    let mut buffer = OutputBuffer::default();
    buffer.clear();

    // Mark only specific lines as changed
    buffer.update_line(2);
    buffer.update_line(5);
    buffer.update_line(7);

    let rows: Vec<Row> = (0..10)
        .map(|_| {
            let mut columns = VecDeque::new();
            columns.push_back(TerminalCharacter::new('A'));
            Row::from_columns(columns)
        })
        .collect();

    let chunks = buffer.changed_chunks_in_viewport(&rows, 10, 10, 0, 0);

    assert_eq!(chunks.len(), 3, "Should return only changed lines");
    assert_eq!(chunks[0].y, 2, "First chunk should be at line 2");
    assert_eq!(chunks[1].y, 5, "Second chunk should be at line 5");
    assert_eq!(chunks[2].y, 7, "Third chunk should be at line 7");
}

#[test]
fn test_pane_defaults_preserved_when_middle_covered() {
    let pane_geom = create_pane_geom(5, 5, 3, 1);
    let stack = FloatingPanesStack {
        layers: vec![pane_geom],
    };

    let pane_bg = Some(AnsiCode::RgbCode((0, 26, 58)));
    let pane_fg = Some(AnsiCode::RgbCode((0, 224, 0)));

    let mut chunk = create_character_chunk_from_str("0123456789", 0, 5);
    chunk.pane_default_bg = pane_bg;
    chunk.pane_default_fg = pane_fg;

    let visible = stack
        .visible_character_chunks(vec![chunk], Some(0))
        .unwrap();

    assert_eq!(visible.len(), 2, "Middle split should produce two chunks");
    for (i, chunk) in visible.iter().enumerate() {
        assert_eq!(
            chunk.pane_default_bg, pane_bg,
            "Chunk {i} should preserve pane_default_bg"
        );
        assert_eq!(
            chunk.pane_default_fg, pane_fg,
            "Chunk {i} should preserve pane_default_fg"
        );
    }
}

#[test]
fn test_pane_defaults_preserved_when_left_covered() {
    let pane_geom = create_pane_geom(0, 5, 5, 1);
    let stack = FloatingPanesStack {
        layers: vec![pane_geom],
    };

    let pane_bg = Some(AnsiCode::RgbCode((0, 26, 58)));

    let mut chunk = create_character_chunk_from_str("0123456789", 0, 5);
    chunk.pane_default_bg = pane_bg;

    let visible = stack
        .visible_character_chunks(vec![chunk], Some(0))
        .unwrap();

    assert_eq!(visible.len(), 1, "Left-covered should produce one chunk");
    assert_eq!(
        visible[0].pane_default_bg, pane_bg,
        "Remaining chunk should preserve pane_default_bg"
    );
}

#[test]
fn test_pane_defaults_preserved_when_right_covered() {
    let pane_geom = create_pane_geom(5, 5, 10, 1);
    let stack = FloatingPanesStack {
        layers: vec![pane_geom],
    };

    let pane_bg = Some(AnsiCode::RgbCode((0, 26, 58)));

    let mut chunk = create_character_chunk_from_str("0123456789", 0, 5);
    chunk.pane_default_bg = pane_bg;

    let visible = stack
        .visible_character_chunks(vec![chunk], Some(0))
        .unwrap();

    assert_eq!(visible.len(), 1, "Right-covered should produce one chunk");
    assert_eq!(
        visible[0].pane_default_bg, pane_bg,
        "Remaining chunk should preserve pane_default_bg"
    );
}
