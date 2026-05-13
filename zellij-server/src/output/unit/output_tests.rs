use super::super::image_fragment::{visible_image_fragments, ImageFragment};
use super::super::kitty_diff::{
    plan_kitty_scene, KittyAssetOp, KittyPlacementKey, KittyPlacementOp, KittyScenePlan,
    KittySceneState, PlannedKittyPlacement,
};
use super::super::{
    CharacterChunk, FloatingPanesStack, KittyFileOutputAcknowledgementPolicy, KittyImageChunk,
    KittyImageData, KittyOutputMediaCache, KittyOutputMediaRetention, KittyPlaceholderCellRender,
    LastRenderedImageState, Output, OutputBuffer, PaneImageRenderOutput, PlacementId,
    RenderedImageState, SixelImageChunk,
};
use crate::panes::kitty_asset_store::{
    KittyAssetData, KittyAssetFormat, KittyAssetStore, KittyByteRange, KittyExternalMedia,
    KittyExternalMediaLocation,
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

fn pid(value: u32) -> PlacementId {
    PlacementId::Protocol(value)
}

fn wire_pid_for_stable_render_id(stable_render_id: u64) -> PlacementId {
    PlacementId::Synthetic(stable_render_id as u32)
}

fn placeholder_wire_pid_for_stable_render_id(stable_render_id: u64) -> PlacementId {
    PlacementId::Synthetic(stable_render_id as u32)
}

/// Helper to create a simple Output instance for testing
fn create_test_output() -> Output {
    let (output, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_test_output_with_state();
    output
}

fn create_test_output_with_state() -> (
    Output,
    Rc<RefCell<SixelImageStore>>,
    Rc<RefCell<KittyAssetStore>>,
    Rc<RefCell<Option<SizeInPixels>>>,
) {
    create_test_output_with_media_cache(Rc::new(RefCell::new(KittyOutputMediaCache::disabled())))
}

fn create_test_output_with_media_cache(
    kitty_output_media_cache: Rc<RefCell<KittyOutputMediaCache>>,
) -> (
    Output,
    Rc<RefCell<SixelImageStore>>,
    Rc<RefCell<KittyAssetStore>>,
    Rc<RefCell<Option<SizeInPixels>>>,
) {
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
    for image_id in 1..=255 {
        seed_test_kitty_asset(
            kitty_asset_store.clone(),
            image_id,
            create_kitty_image_data(image_id),
        );
    }
    let character_cell_size = Rc::new(RefCell::new(Some(SizeInPixels {
        height: 20,
        width: 10,
    })));
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    (
        Output::new(
            sixel_image_store.clone(),
            kitty_asset_store.clone(),
            kitty_output_media_cache,
            character_cell_size.clone(),
            styled_underlines,
            osc8_hyperlinks,
        ),
        sixel_image_store,
        kitty_asset_store,
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
        stable_render_id: image_id as u64 * 2 - 1,
        image_id,
        placement_id: Some(pid(image_id)),
        cell_x: 0,
        cell_y: 0,
        columns,
        rows,
        columns_specified: true,
        rows_specified: true,
        source_x: 0,
        source_y: 0,
        source_width: 10,
        source_height: 10,
        z_index: 0,
        x_offset: 0,
        y_offset: 0,
    }
}

fn create_rendered_image_state(
    explicit_chunks: Vec<KittyImageChunk>,
    placeholder_renders: Vec<crate::output::KittyPlaceholderRender>,
) -> RenderedImageState {
    RenderedImageState {
        explicit_chunks,
        placeholder_renders,
        resident_asset_generations: HashMap::new(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TestRect {
    x: usize,
    y: usize,
    columns: usize,
    rows: usize,
}

fn kitty_asset_op_sort_key(op: &KittyAssetOp) -> (u32, u64) {
    match op {
        KittyAssetOp::EnsureResident {
            image_id,
            generation,
        } => (*image_id, *generation),
    }
}

fn kitty_placement_op_sort_key(op: &KittyPlacementOp) -> (u8, KittyPlacementKey) {
    match op {
        KittyPlacementOp::Delete { key } => (0, *key),
        KittyPlacementOp::PlaceExplicit { key, .. }
        | KittyPlacementOp::PlacePlaceholder { key, .. } => (1, *key),
    }
}

fn assert_kitty_scene_plan_eq_unordered(actual: KittyScenePlan, expected: KittyScenePlan) {
    match (actual, expected) {
        (
            KittyScenePlan::Diff {
                asset_ops: mut actual_asset_ops,
                placement_ops: mut actual_placement_ops,
            },
            KittyScenePlan::Diff {
                asset_ops: mut expected_asset_ops,
                placement_ops: mut expected_placement_ops,
            },
        ) => {
            actual_asset_ops.sort_by_key(kitty_asset_op_sort_key);
            expected_asset_ops.sort_by_key(kitty_asset_op_sort_key);
            actual_placement_ops.sort_by_key(kitty_placement_op_sort_key);
            expected_placement_ops.sort_by_key(kitty_placement_op_sort_key);
            assert_eq!(actual_asset_ops, expected_asset_ops);
            assert_eq!(actual_placement_ops, expected_placement_ops);
        },
        (actual, expected) => assert_eq!(actual, expected),
    }
}

fn test_scale_u32(total: u32, kept: usize, original: usize) -> u32 {
    if original == 0 {
        0
    } else {
        ((total as u64 * kept as u64) / original as u64) as u32
    }
}

fn rect_intersection(a: TestRect, b: TestRect) -> Option<TestRect> {
    let left = a.x.max(b.x);
    let top = a.y.max(b.y);
    let right = (a.x + a.columns).min(b.x + b.columns);
    let bottom = (a.y + a.rows).min(b.y + b.rows);
    if left < right && top < bottom {
        Some(TestRect {
            x: left,
            y: top,
            columns: right - left,
            rows: bottom - top,
        })
    } else {
        None
    }
}

fn subtract_rect(rect: TestRect, occluder: TestRect) -> Vec<TestRect> {
    let Some(intersection) = rect_intersection(rect, occluder) else {
        return vec![rect];
    };
    if intersection == rect {
        return vec![];
    }
    let mut fragments = vec![];
    if intersection.y > rect.y {
        fragments.push(TestRect {
            x: rect.x,
            y: rect.y,
            columns: rect.columns,
            rows: intersection.y - rect.y,
        });
    }
    let rect_bottom = rect.y + rect.rows;
    let intersection_bottom = intersection.y + intersection.rows;
    if intersection_bottom < rect_bottom {
        fragments.push(TestRect {
            x: rect.x,
            y: intersection_bottom,
            columns: rect.columns,
            rows: rect_bottom - intersection_bottom,
        });
    }
    if intersection.x > rect.x {
        fragments.push(TestRect {
            x: rect.x,
            y: intersection.y,
            columns: intersection.x - rect.x,
            rows: intersection.rows,
        });
    }
    let rect_right = rect.x + rect.columns;
    let intersection_right = intersection.x + intersection.columns;
    if intersection_right < rect_right {
        fragments.push(TestRect {
            x: intersection_right,
            y: intersection.y,
            columns: rect_right - intersection_right,
            rows: intersection.rows,
        });
    }
    fragments
}

fn expected_explicit_fragments_for_occluders(
    chunk: &KittyImageChunk,
    occluders: &[TestRect],
) -> Vec<KittyImageChunk> {
    let mut rects = vec![TestRect {
        x: chunk.cell_x,
        y: chunk.cell_y,
        columns: chunk.columns,
        rows: chunk.rows,
    }];
    for occluder in occluders {
        let mut next = vec![];
        for rect in rects {
            next.extend(subtract_rect(rect, *occluder));
        }
        rects = next;
    }
    let mut chunks = rects
        .into_iter()
        .map(|rect| KittyImageChunk {
            cell_x: rect.x,
            cell_y: rect.y,
            columns: rect.columns,
            rows: rect.rows,
            source_x: chunk.source_x
                + test_scale_u32(chunk.source_width, rect.x - chunk.cell_x, chunk.columns),
            source_y: chunk.source_y
                + test_scale_u32(chunk.source_height, rect.y - chunk.cell_y, chunk.rows),
            source_width: test_scale_u32(chunk.source_width, rect.columns, chunk.columns),
            source_height: test_scale_u32(chunk.source_height, rect.rows, chunk.rows),
            ..chunk.clone()
        })
        .collect::<Vec<_>>();
    chunks.sort_by_key(|chunk| (chunk.cell_y, chunk.cell_x, chunk.rows, chunk.columns));
    chunks
}

fn kitty_chunk_signature(chunk: &KittyImageChunk) -> String {
    format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        chunk.cell_x,
        chunk.cell_y,
        chunk.columns,
        chunk.rows,
        chunk.columns_specified,
        chunk.rows_specified,
        chunk.source_x,
        chunk.source_y,
        chunk.source_width,
        chunk.source_height,
        chunk.z_index,
        chunk.x_offset,
        chunk.y_offset
    )
}

fn assert_kitty_chunk_sets_eq(actual: &[KittyImageChunk], expected: &[KittyImageChunk]) {
    let mut actual = actual.iter().map(kitty_chunk_signature).collect::<Vec<_>>();
    let mut expected = expected
        .iter()
        .map(kitty_chunk_signature)
        .collect::<Vec<_>>();
    actual.sort_unstable();
    expected.sort_unstable();
    assert_eq!(actual, expected);
}

fn create_kitty_image_data(image_id: u32) -> KittyImageData {
    let byte = image_id as u8;
    KittyImageData::Png {
        data: vec![
            byte,
            byte.wrapping_add(1),
            byte.wrapping_add(2),
            byte.wrapping_add(3),
        ],
        width: 1,
        height: 1,
    }
}

fn create_kitty_generation(_image_id: u32) -> u64 {
    1
}

fn create_kitty_placeholder_render(image_id: u32) -> crate::output::KittyPlaceholderRender {
    crate::output::KittyPlaceholderRender {
        stable_render_id: image_id as u64 * 2,
        image_id,
        placement_id: Some(pid(image_id)),
        columns: 1,
        rows: 1,
        source_x: 0,
        source_y: 0,
        source_width: 10,
        source_height: 10,
        x_offset: 0,
        y_offset: 0,
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
    chunk.placement_id = Some(pid(placement_id));
    chunk.stable_render_id = placement_id as u64;
    PlannedKittyPlacement::Explicit {
        key: KittyPlacementKey {
            stable_render_id: placement_id as u64,
            image_id,
            wire_placement_id: pid(placement_id),
        },
        chunk,
    }
}

fn create_kitty_diff_placeholder_placement(
    image_id: u32,
    placement_id: u32,
) -> PlannedKittyPlacement {
    let mut render = create_kitty_placeholder_render(image_id);
    render.placement_id = Some(pid(placement_id));
    render.stable_render_id = placement_id as u64;
    PlannedKittyPlacement::Placeholder {
        key: KittyPlacementKey {
            stable_render_id: placement_id as u64,
            image_id,
            wire_placement_id: pid(placement_id),
        },
        render,
    }
}

fn create_kitty_scene_state(placements: Vec<PlannedKittyPlacement>) -> KittySceneState {
    let mut scene = KittySceneState::default();
    for placement in placements {
        let image_id = placement.key().image_id;
        scene.insert_asset(image_id, create_kitty_generation(image_id));
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

fn seed_test_kitty_asset(
    kitty_asset_store: Rc<RefCell<KittyAssetStore>>,
    image_id: u32,
    image_data: KittyImageData,
) {
    kitty_asset_store
        .borrow_mut()
        .insert_asset(image_id, image_data);
}

fn seed_test_file_backed_kitty_asset(
    kitty_asset_store: Rc<RefCell<KittyAssetStore>>,
    image_id: u32,
    media: KittyExternalMedia,
) {
    kitty_asset_store.borrow_mut().insert_asset_data_protecting(
        image_id,
        KittyAssetData::External {
            media,
            format: KittyAssetFormat::Rgba,
            width: 2,
            height: 2,
        },
        &HashSet::new(),
    );
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

    assert_kitty_scene_plan_eq_unordered(
        plan,
        KittyScenePlan::Diff {
            asset_ops: vec![],
            placement_ops: vec![],
        },
    );
}

#[test]
fn test_kitty_diff_deletes_removed_placement() {
    let assumed = create_kitty_scene_state(vec![create_kitty_diff_explicit_placement(1, 10, 2, 2)]);
    let desired = KittySceneState::default();

    let plan = plan_kitty_scene(&assumed, &desired);

    assert_kitty_scene_plan_eq_unordered(
        plan,
        KittyScenePlan::Diff {
            asset_ops: vec![],
            placement_ops: vec![KittyPlacementOp::Delete {
                key: KittyPlacementKey {
                    stable_render_id: 10,
                    image_id: 1,
                    wire_placement_id: pid(10),
                },
            }],
        },
    );
}

#[test]
fn test_kitty_diff_places_resident_asset_without_retransmit() {
    let mut assumed = KittySceneState::default();
    assumed
        .resident_asset_generations
        .insert(1, create_kitty_generation(1));
    let desired = create_kitty_scene_state(vec![create_kitty_diff_explicit_placement(1, 10, 2, 2)]);

    let plan = plan_kitty_scene(&assumed, &desired);

    assert_kitty_scene_plan_eq_unordered(
        plan,
        KittyScenePlan::Diff {
            asset_ops: vec![],
            placement_ops: vec![KittyPlacementOp::PlaceExplicit {
                key: KittyPlacementKey {
                    stable_render_id: 10,
                    image_id: 1,
                    wire_placement_id: pid(10),
                },
                chunk: {
                    let mut chunk = create_kitty_chunk(1, 2, 2);
                    chunk.placement_id = Some(pid(10));
                    chunk.stable_render_id = 10;
                    chunk
                },
            }],
        },
    );
}

#[test]
fn test_kitty_diff_retransmits_missing_asset_before_place() {
    let assumed = KittySceneState::default();
    let desired = create_kitty_scene_state(vec![create_kitty_diff_explicit_placement(1, 10, 2, 2)]);

    let plan = plan_kitty_scene(&assumed, &desired);

    assert_kitty_scene_plan_eq_unordered(
        plan,
        KittyScenePlan::Diff {
            asset_ops: vec![KittyAssetOp::EnsureResident {
                image_id: 1,
                generation: create_kitty_generation(1),
            }],
            placement_ops: vec![KittyPlacementOp::PlaceExplicit {
                key: KittyPlacementKey {
                    stable_render_id: 10,
                    image_id: 1,
                    wire_placement_id: pid(10),
                },
                chunk: {
                    let mut chunk = create_kitty_chunk(1, 2, 2);
                    chunk.placement_id = Some(pid(10));
                    chunk.stable_render_id = 10;
                    chunk
                },
            }],
        },
    );
}

#[test]
fn test_kitty_diff_replaces_geometry_change_with_delete_and_place() {
    let assumed = create_kitty_scene_state(vec![create_kitty_diff_explicit_placement(1, 10, 2, 2)]);
    let desired = create_kitty_scene_state(vec![create_kitty_diff_explicit_placement(1, 10, 3, 2)]);

    let plan = plan_kitty_scene(&assumed, &desired);

    assert_kitty_scene_plan_eq_unordered(
        plan,
        KittyScenePlan::Diff {
            asset_ops: vec![],
            placement_ops: vec![
                KittyPlacementOp::Delete {
                    key: KittyPlacementKey {
                        stable_render_id: 10,
                        image_id: 1,
                        wire_placement_id: pid(10),
                    },
                },
                KittyPlacementOp::PlaceExplicit {
                    key: KittyPlacementKey {
                        stable_render_id: 10,
                        image_id: 1,
                        wire_placement_id: pid(10),
                    },
                    chunk: {
                        let mut chunk = create_kitty_chunk(1, 3, 2);
                        chunk.placement_id = Some(pid(10));
                        chunk.stable_render_id = 10;
                        chunk
                    },
                },
            ],
        },
    );
}

#[test]
fn test_kitty_diff_invalidates_all_placements_when_asset_payload_changes() {
    let assumed = create_kitty_scene_state(vec![
        create_kitty_diff_explicit_placement(1, 10, 2, 2),
        create_kitty_diff_explicit_placement(1, 11, 2, 2),
    ]);
    let mut changed_chunk = create_kitty_chunk(1, 2, 2);
    changed_chunk.placement_id = Some(pid(10));
    changed_chunk.stable_render_id = 10;
    let mut changed_chunk_two = changed_chunk.clone();
    changed_chunk_two.placement_id = Some(pid(11));
    changed_chunk_two.stable_render_id = 11;
    let mut desired = create_kitty_scene_state(vec![
        PlannedKittyPlacement::Explicit {
            key: KittyPlacementKey {
                stable_render_id: 10,
                image_id: 1,
                wire_placement_id: pid(10),
            },
            chunk: changed_chunk.clone(),
        },
        PlannedKittyPlacement::Explicit {
            key: KittyPlacementKey {
                stable_render_id: 11,
                image_id: 1,
                wire_placement_id: pid(11),
            },
            chunk: changed_chunk_two.clone(),
        },
    ]);
    desired.insert_asset(1, create_kitty_generation(1) + 1);

    let plan = plan_kitty_scene(&assumed, &desired);

    assert_kitty_scene_plan_eq_unordered(
        plan,
        KittyScenePlan::Diff {
            asset_ops: vec![KittyAssetOp::EnsureResident {
                image_id: 1,
                generation: create_kitty_generation(1) + 1,
            }],
            placement_ops: vec![
                KittyPlacementOp::Delete {
                    key: KittyPlacementKey {
                        stable_render_id: 10,
                        image_id: 1,
                        wire_placement_id: pid(10),
                    },
                },
                KittyPlacementOp::Delete {
                    key: KittyPlacementKey {
                        stable_render_id: 11,
                        image_id: 1,
                        wire_placement_id: pid(11),
                    },
                },
                KittyPlacementOp::PlaceExplicit {
                    key: KittyPlacementKey {
                        stable_render_id: 10,
                        image_id: 1,
                        wire_placement_id: pid(10),
                    },
                    chunk: changed_chunk,
                },
                KittyPlacementOp::PlaceExplicit {
                    key: KittyPlacementKey {
                        stable_render_id: 11,
                        image_id: 1,
                        wire_placement_id: pid(11),
                    },
                    chunk: changed_chunk_two,
                },
            ],
        },
    );
}

#[test]
fn test_kitty_diff_shared_asset_updates_explicit_and_placeholder_placements() {
    let assumed = create_kitty_scene_state(vec![
        create_kitty_diff_explicit_placement(1, 10, 2, 2),
        create_kitty_diff_placeholder_placement(1, 20),
    ]);
    let mut changed_chunk = create_kitty_chunk(1, 2, 2);
    changed_chunk.placement_id = Some(pid(10));
    changed_chunk.stable_render_id = 10;
    let mut changed_render = create_kitty_placeholder_render(1);
    changed_render.placement_id = Some(pid(20));
    changed_render.stable_render_id = 20;
    let mut desired = create_kitty_scene_state(vec![
        PlannedKittyPlacement::Explicit {
            key: KittyPlacementKey {
                stable_render_id: 10,
                image_id: 1,
                wire_placement_id: pid(10),
            },
            chunk: changed_chunk.clone(),
        },
        PlannedKittyPlacement::Placeholder {
            key: KittyPlacementKey {
                stable_render_id: 20,
                image_id: 1,
                wire_placement_id: pid(20),
            },
            render: changed_render.clone(),
        },
    ]);
    desired.insert_asset(1, create_kitty_generation(1) + 1);

    let plan = plan_kitty_scene(&assumed, &desired);

    assert_kitty_scene_plan_eq_unordered(
        plan,
        KittyScenePlan::Diff {
            asset_ops: vec![KittyAssetOp::EnsureResident {
                image_id: 1,
                generation: create_kitty_generation(1) + 1,
            }],
            placement_ops: vec![
                KittyPlacementOp::Delete {
                    key: KittyPlacementKey {
                        stable_render_id: 10,
                        image_id: 1,
                        wire_placement_id: pid(10),
                    },
                },
                KittyPlacementOp::Delete {
                    key: KittyPlacementKey {
                        stable_render_id: 20,
                        image_id: 1,
                        wire_placement_id: pid(20),
                    },
                },
                KittyPlacementOp::PlaceExplicit {
                    key: KittyPlacementKey {
                        stable_render_id: 10,
                        image_id: 1,
                        wire_placement_id: pid(10),
                    },
                    chunk: changed_chunk,
                },
                KittyPlacementOp::PlacePlaceholder {
                    key: KittyPlacementKey {
                        stable_render_id: 20,
                        image_id: 1,
                        wire_placement_id: pid(20),
                    },
                    render: changed_render,
                },
            ],
        },
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
    unchanged_output.set_last_rendered_image_states(HashMap::from([(
        1,
        create_rendered_image_state(vec![base_chunk.clone()], vec![]),
    )]));
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
    changed_output.set_last_rendered_image_states(HashMap::from([(
        1,
        create_rendered_image_state(vec![base_chunk.clone()], vec![]),
    )]));
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
    cleared_output.set_last_rendered_image_states(HashMap::from([(
        1,
        create_rendered_image_state(vec![base_chunk], vec![]),
    )]));
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
    output.set_last_rendered_image_states(HashMap::from([(
        1,
        create_rendered_image_state(vec![base_chunk.clone()], vec![]),
    )]));
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
    first_chunk.placement_id = Some(pid(10));
    let mut second_chunk = create_kitty_chunk(2, 2, 2);
    second_chunk.placement_id = Some(pid(20));
    second_chunk.cell_x = 5;

    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.set_last_rendered_image_states(HashMap::from([(
        1,
        create_rendered_image_state(vec![first_chunk.clone()], vec![]),
    )]));
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
    first_chunk.placement_id = Some(pid(10));
    let mut second_chunk = create_kitty_chunk(2, 2, 2);
    second_chunk.placement_id = Some(pid(20));
    second_chunk.cell_x = 5;

    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.set_last_rendered_image_states(HashMap::from([(
        1,
        create_rendered_image_state(vec![first_chunk.clone(), second_chunk], vec![]),
    )]));
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
    first_chunk.placement_id = Some(pid(10));
    let mut second_chunk = create_kitty_chunk(2, 2, 2);
    second_chunk.placement_id = Some(pid(20));
    second_chunk.cell_x = 5;

    let mut changed_second_chunk = second_chunk.clone();
    changed_second_chunk.columns = 3;

    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.set_last_rendered_image_states(HashMap::from([(
        1,
        create_rendered_image_state(vec![first_chunk.clone(), second_chunk], vec![]),
    )]));
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
    first_chunk.placement_id = Some(pid(10));
    let mut second_chunk = create_kitty_chunk(1, 2, 2);
    second_chunk.placement_id = Some(pid(11));
    second_chunk.cell_x = 5;
    let mut other_asset_chunk = create_kitty_chunk(2, 2, 2);
    other_asset_chunk.placement_id = Some(pid(20));
    other_asset_chunk.cell_x = 10;
    let (mut output, _sixel_image_store, kitty_asset_store, _character_cell_size) =
        create_test_output_with_state();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(vec![
            first_chunk.clone(),
            second_chunk.clone(),
            other_asset_chunk.clone(),
        ]),
        None,
    );
    let _ = output.serialize().unwrap();

    seed_test_kitty_asset(
        kitty_asset_store,
        1,
        KittyImageData::Png {
            data: vec![7, 7, 7, 7],
            width: 1,
            height: 1,
        },
    );
    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(vec![first_chunk, second_chunk, other_asset_chunk]),
        None,
    );

    let serialized = output.serialize().unwrap();
    let client_output = serialized.get(&1).unwrap();
    assert!(
        client_output.contains("a=t"),
        "asset changes should retransmit updated kitty image bytes"
    );
    assert!(
        !client_output.contains("\u{1b}_Ga=d,d=A\u{1b}\\"),
        "asset changes should not force a kitty delete-all when per-asset updates suffice"
    );
}

#[test]
fn test_image_output_can_publish_resident_assets_as_regular_files() {
    let tempdir = tempfile::tempdir().unwrap();
    let media_dir = tempdir.path().join("session-media/test/image");
    let media_cache = Rc::new(RefCell::new(KittyOutputMediaCache::new(media_dir.clone())));
    let client_ids = create_test_clients(1);
    let chunk = create_kitty_chunk(1, 2, 2);
    let (mut output, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_test_output_with_media_cache(media_cache);
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.set_kitty_file_output_enabled_for_client(1, true);
    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(vec![chunk]),
        None,
    );

    let serialized = output.serialize().unwrap();
    let client_output = serialized.get(&1).unwrap();
    assert!(
        client_output.contains("a=t,i=1,q=2,f=100,t=f;"),
        "file-enabled clients should receive regular-file kitty uploads"
    );
    assert!(
        !client_output.contains("AQIDBA=="),
        "file-enabled clients should not receive the direct base64 image payload"
    );

    let files = std::fs::read_dir(&media_dir)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(std::fs::read(files[0].path()).unwrap(), vec![1, 2, 3, 4]);
}

#[test]
fn test_image_output_publishes_file_backed_raw_assets_as_regular_files() {
    let tempdir = tempfile::tempdir().unwrap();
    let media_dir = tempdir.path().join("session-media/test/image");
    let source_path = tempdir.path().join("source-rgba.bin");
    let payload = vec![
        255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
    ];
    std::fs::write(&source_path, &payload).unwrap();
    let media_cache = Rc::new(RefCell::new(KittyOutputMediaCache::new(media_dir.clone())));
    let client_ids = create_test_clients(1);
    let chunk = create_kitty_chunk(88, 2, 2);
    let (mut output, _sixel_image_store, kitty_asset_store, _character_cell_size) =
        create_test_output_with_media_cache(media_cache);
    seed_test_file_backed_kitty_asset(
        kitty_asset_store,
        88,
        KittyExternalMedia::new(
            KittyExternalMediaLocation::RegularFile(source_path),
            KittyByteRange {
                offset: 0,
                size: Some(payload.len()),
            },
        ),
    );
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.set_kitty_file_output_enabled_for_client(1, true);
    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(vec![chunk]),
        None,
    );

    let serialized = output.serialize().unwrap();
    let client_output = serialized.get(&1).unwrap();
    assert!(
        client_output.contains("a=t,i=88,q=2,f=32,s=2,v=2,t=f;"),
        "file-backed raw assets should be sent through kitty file transport"
    );
    assert!(
        !client_output.contains(&base64::encode(&payload)),
        "file-backed raw assets should not be serialized as direct base64 payloads"
    );

    let files = std::fs::read_dir(&media_dir)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(std::fs::read(files[0].path()).unwrap(), payload);
}

#[test]
fn test_image_output_file_transport_is_per_client_and_reuses_published_files() {
    let tempdir = tempfile::tempdir().unwrap();
    let media_dir = tempdir.path().join("session-media/test/image");
    let media_cache = Rc::new(RefCell::new(KittyOutputMediaCache::new(media_dir.clone())));
    let client_ids = create_test_clients(2);
    let chunk = create_kitty_chunk(1, 2, 2);
    let (mut output, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_test_output_with_media_cache(media_cache);
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.set_kitty_file_output_enabled_for_client(1, true);
    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(vec![chunk.clone()]),
        None,
    );
    output.add_pane_image_output_to_client(
        2,
        pane_image_output_with_kitty_scene(vec![chunk]),
        None,
    );

    let serialized = output.serialize().unwrap();
    let file_client_output = serialized.get(&1).unwrap();
    let direct_client_output = serialized.get(&2).unwrap();
    assert!(file_client_output.contains("a=t,i=1,q=2,f=100,t=f;"));
    assert!(!file_client_output.contains("AQIDBA=="));
    assert!(!direct_client_output.contains("t=f;"));
    assert!(direct_client_output.contains("AQIDBA=="));

    let files = std::fs::read_dir(&media_dir)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(files.len(), 1);
}

#[test]
fn test_image_output_can_request_acknowledgements_for_file_transport() {
    let tempdir = tempfile::tempdir().unwrap();
    let media_dir = tempdir.path().join("session-media/test/image");
    let media_cache = Rc::new(RefCell::new(KittyOutputMediaCache::new(media_dir.clone())));
    let client_ids = create_test_clients(1);
    let chunk = create_kitty_chunk(1, 2, 2);
    let (mut output, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_test_output_with_media_cache(media_cache);
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.set_kitty_file_output_enabled_for_client(1, true);
    output.set_kitty_file_output_acknowledgement_policy_for_client(
        1,
        KittyFileOutputAcknowledgementPolicy::Always,
    );
    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(vec![chunk]),
        None,
    );

    let serialized = output.serialize().unwrap();
    let client_output = serialized.get(&1).unwrap();
    assert!(
        client_output.contains("a=t,i=1,q=0,f=100,t=f;"),
        "ack-enabled file uploads should request a terminal reply"
    );
}

#[test]
fn test_image_output_watermark_requests_acknowledgement_only_for_last_file_upload() {
    let tempdir = tempfile::tempdir().unwrap();
    let media_dir = tempdir.path().join("session-media/test/image");
    let media_cache = Rc::new(RefCell::new(KittyOutputMediaCache::new(media_dir)));
    let client_ids = create_test_clients(1);
    let first_chunk = create_kitty_chunk(1, 2, 2);
    let second_chunk = create_kitty_chunk(2, 2, 2);
    let (mut output, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_test_output_with_media_cache(media_cache);
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.set_kitty_file_output_enabled_for_client(1, true);
    output.set_kitty_file_output_acknowledgement_policy_for_client(
        1,
        KittyFileOutputAcknowledgementPolicy::Watermark,
    );
    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(vec![first_chunk, second_chunk]),
        None,
    );

    let serialized = output.serialize().unwrap();
    let client_output = serialized.get(&1).unwrap();
    let tracked_uploads = client_output.matches("q=2,f=100,t=f;").count();
    let requested_uploads = client_output.matches("q=0,f=100,t=f;").count();
    assert!(
        tracked_uploads == 1,
        "watermark should track exactly one earlier file upload without requesting replies, got {client_output:?}"
    );
    assert!(
        requested_uploads == 1,
        "watermark should request exactly one reply for the last file upload, got {client_output:?}"
    );
}

#[test]
fn test_output_media_cache_grace_retention_keeps_recent_files() {
    let tempdir = tempfile::tempdir().unwrap();
    let media_dir = tempdir.path().join("session-media/test/image");
    let mut media_cache = KittyOutputMediaCache::new(media_dir);
    let image_data = KittyImageData::Png {
        data: vec![1, 2, 3, 4],
        width: 1,
        height: 1,
    };

    let media_path = media_cache
        .ensure_regular_file(1, 1, &image_data)
        .expect("should write test media file");
    media_cache.retain_files(KittyOutputMediaRetention::KeepRecentlyReferenced, |_, _| {
        false
    });

    assert!(
        media_path.exists(),
        "grace retention should keep recently referenced media even when not explicitly live"
    );
}

#[test]
fn test_output_media_cache_explicit_retention_reaps_unkept_files() {
    let tempdir = tempfile::tempdir().unwrap();
    let media_dir = tempdir.path().join("session-media/test/image");
    let mut media_cache = KittyOutputMediaCache::new(media_dir);
    let image_data = KittyImageData::Png {
        data: vec![1, 2, 3, 4],
        width: 1,
        height: 1,
    };

    let media_path = media_cache
        .ensure_regular_file(1, 1, &image_data)
        .expect("should write test media file");
    media_cache.retain_files(KittyOutputMediaRetention::OnlyExplicitlyKept, |_, _| false);

    assert!(
        !media_path.exists(),
        "explicit-only retention should remove media that the caller does not keep"
    );
}

#[test]
fn test_output_media_cache_explicit_retention_keeps_pending_file_reads_until_acknowledged() {
    let tempdir = tempfile::tempdir().unwrap();
    let media_dir = tempdir.path().join("session-media/test/image");
    let mut media_cache = KittyOutputMediaCache::new(media_dir);
    let image_data = KittyImageData::Png {
        data: vec![1, 2, 3, 4],
        width: 1,
        height: 1,
    };

    let media_path = media_cache
        .ensure_regular_file(1, 1, &image_data)
        .expect("should write test media file");
    media_cache.mark_pending_regular_file_read(1, 1, 1, true);
    media_cache.retain_files(KittyOutputMediaRetention::OnlyExplicitlyKept, |_, _| false);
    assert!(
        media_path.exists(),
        "pending file reads should keep media alive without a grace window"
    );

    media_cache.acknowledge_regular_file_read(1, 1);
    media_cache.retain_files(KittyOutputMediaRetention::OnlyExplicitlyKept, |_, _| false);
    assert!(
        !media_path.exists(),
        "acknowledged file reads should no longer retain media"
    );
}

#[test]
fn test_output_media_cache_watermark_acknowledgement_releases_earlier_pending_reads() {
    let tempdir = tempfile::tempdir().unwrap();
    let media_dir = tempdir.path().join("session-media/test/image");
    let mut media_cache = KittyOutputMediaCache::new(media_dir);
    let image_data = KittyImageData::Png {
        data: vec![1, 2, 3, 4],
        width: 1,
        height: 1,
    };

    let first_path = media_cache
        .ensure_regular_file(1, 1, &image_data)
        .expect("should write first media file");
    let second_path = media_cache
        .ensure_regular_file(2, 1, &image_data)
        .expect("should write second media file");
    media_cache.mark_pending_regular_file_read(1, 1, 1, false);
    media_cache.mark_pending_regular_file_read(1, 2, 1, true);
    media_cache.retain_files(KittyOutputMediaRetention::OnlyExplicitlyKept, |_, _| false);
    assert!(first_path.exists());
    assert!(second_path.exists());

    media_cache.acknowledge_regular_file_read(1, 2);
    media_cache.retain_files(KittyOutputMediaRetention::OnlyExplicitlyKept, |_, _| false);
    assert!(
        !first_path.exists() && !second_path.exists(),
        "watermark ack should release all pending file reads through the acknowledged upload"
    );
}

#[test]
fn test_output_media_cache_rename_failure_keeps_old_files_tracked_for_cleanup() {
    let old_session_name = format!("zellij-test-kitty-rename-fail-old-{}", std::process::id());
    let new_session_name = format!("zellij-test-kitty-rename-fail-new-{}", std::process::id());
    KittyOutputMediaCache::cleanup_session_media(&old_session_name);
    KittyOutputMediaCache::cleanup_session_media(&new_session_name);

    let mut media_cache = KittyOutputMediaCache::new_for_session(&old_session_name);
    let image_data = KittyImageData::Png {
        data: vec![1, 2, 3, 4],
        width: 1,
        height: 1,
    };
    let media_path = media_cache
        .ensure_regular_file(1, 1, &image_data)
        .expect("should write test media file");
    let old_session_dir = media_path
        .parent()
        .and_then(|path| path.parent())
        .expect("media path should be inside the session image dir")
        .to_path_buf();
    let new_session_dir = old_session_dir
        .parent()
        .expect("old session dir should have a parent")
        .join(&new_session_name);
    std::fs::create_dir_all(&new_session_dir).expect("should create conflicting session dir");
    std::fs::write(new_session_dir.join("conflict"), b"conflict")
        .expect("should make conflicting session dir non-empty");

    media_cache.mark_pending_regular_file_read(1, 1, 1, true);
    media_cache.rename_session(&old_session_name, &new_session_name);
    media_cache.acknowledge_regular_file_read(1, 1);
    media_cache.retain_files(KittyOutputMediaRetention::OnlyExplicitlyKept, |_, _| false);

    assert!(
        !media_path.exists(),
        "failed rename should keep old media files tracked so later retention can clean them up"
    );

    KittyOutputMediaCache::cleanup_session_media(&old_session_name);
    KittyOutputMediaCache::cleanup_session_media(&new_session_name);
}

#[test]
fn test_output_media_cache_successful_rename_drops_stale_pending_file_reads() {
    let old_session_name = format!(
        "zellij-test-kitty-rename-success-old-{}",
        std::process::id()
    );
    let new_session_name = format!(
        "zellij-test-kitty-rename-success-new-{}",
        std::process::id()
    );
    KittyOutputMediaCache::cleanup_session_media(&old_session_name);
    KittyOutputMediaCache::cleanup_session_media(&new_session_name);

    let mut media_cache = KittyOutputMediaCache::new_for_session(&old_session_name);
    let image_data = KittyImageData::Png {
        data: vec![1, 2, 3, 4],
        width: 1,
        height: 1,
    };
    media_cache
        .ensure_regular_file(1, 1, &image_data)
        .expect("should write old test media file");
    media_cache.mark_pending_regular_file_read(1, 1, 1, true);

    media_cache.rename_session(&old_session_name, &new_session_name);
    let new_media_path = media_cache
        .ensure_regular_file(1, 1, &image_data)
        .expect("should write new test media file");
    media_cache.retain_files(KittyOutputMediaRetention::OnlyExplicitlyKept, |_, _| false);

    assert!(
        !new_media_path.exists(),
        "stale pending reads from before a successful rename should not pin new media files"
    );

    KittyOutputMediaCache::cleanup_session_media(&old_session_name);
    KittyOutputMediaCache::cleanup_session_media(&new_session_name);
}

#[test]
fn test_image_output_asset_change_recreates_shared_explicit_and_placeholder_placements() {
    let client_ids = create_test_clients(1);
    let mut explicit_chunk = create_kitty_chunk(1, 2, 2);
    explicit_chunk.placement_id = Some(pid(10));
    explicit_chunk.cell_x = 5;
    let mut placeholder_render = create_kitty_placeholder_render(1);
    placeholder_render.placement_id = Some(pid(20));
    let explicit_wire_pid = wire_pid_for_stable_render_id(explicit_chunk.stable_render_id);
    let placeholder_wire_pid =
        placeholder_wire_pid_for_stable_render_id(placeholder_render.stable_render_id);

    let (mut output, _sixel_image_store, kitty_asset_store, _character_cell_size) =
        create_test_output_with_state();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.add_pane_image_output_to_client(
        1,
        PaneImageRenderOutput {
            kitty_scene: crate::output::KittyRenderBundle {
                explicit_chunks: vec![explicit_chunk.clone()],
                placeholder_renders: vec![placeholder_render.clone()],
            },
            ..Default::default()
        },
        None,
    );
    let _ = output.serialize().unwrap();

    seed_test_kitty_asset(
        kitty_asset_store,
        1,
        KittyImageData::Png {
            data: vec![7, 7, 7, 7],
            width: 1,
            height: 1,
        },
    );
    output.add_pane_image_output_to_client(
        1,
        PaneImageRenderOutput {
            kitty_scene: crate::output::KittyRenderBundle {
                explicit_chunks: vec![explicit_chunk],
                placeholder_renders: vec![placeholder_render],
            },
            ..Default::default()
        },
        None,
    );

    let serialized = output.serialize().unwrap();
    let client_output = serialized.get(&1).unwrap();
    assert!(
        client_output.contains(&format!(
            "\u{1b}_Ga=d,d=i,i=1,p={}\u{1b}\\",
            explicit_wire_pid.wire_value()
        )),
        "shared asset replacement should delete the explicit placement before recreating it"
    );
    assert!(
        client_output.contains(&format!(
            "\u{1b}_Ga=d,d=i,i=1,p={}\u{1b}\\",
            placeholder_wire_pid.wire_value()
        )),
        "shared asset replacement should delete the placeholder placement before recreating it"
    );
    assert!(
        client_output.contains(&format!(
            "\u{1b}_Ga=p,i=1,p={}",
            explicit_wire_pid.wire_value()
        )),
        "shared asset replacement should recreate the explicit placement"
    );
    assert!(
        client_output.contains(&format!(
            "\u{1b}_Ga=p,U=1,i=1,p={}",
            placeholder_wire_pid.wire_value()
        )),
        "shared asset replacement should recreate the placeholder placement"
    );
}

#[test]
fn test_image_output_pre_vte_clear_invalidates_assumed_kitty_scene() {
    let client_ids = create_test_clients(1);
    let mut chunk = create_kitty_chunk(1, 2, 2);
    chunk.placement_id = Some(pid(10));

    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.set_last_rendered_image_states(HashMap::from([(
        1,
        create_rendered_image_state(vec![chunk.clone()], vec![]),
    )]));
    output.add_display_clearing_pre_vte_instruction_to_client(1, "\u{1b}[2J");
    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(vec![chunk]),
        None,
    );

    let serialized = output.serialize().unwrap();
    let client_output = serialized.get(&1).unwrap();
    assert!(
        client_output.contains("\u{1b}_Ga=d,d=A\u{1b}\\"),
        "a pre-VTE display clear should also clear host kitty state before rebuilding images"
    );
    assert!(
        client_output.contains("a=p"),
        "after a pre-VTE clear the kitty scene should still be rebuilt"
    );
    assert!(
        client_output.contains("a=t"),
        "after a pre-VTE clear the kitty asset should be retransmitted because 2J invalidates assumed kitty residency"
    );
}

#[test]
fn test_image_output_pane_kitty_clear_resets_host_state_before_rebuild() {
    let client_ids = create_test_clients(1);
    let mut chunk = create_kitty_chunk(1, 2, 2);
    chunk.placement_id = Some(pid(10));

    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.set_last_rendered_image_states(HashMap::from([(
        1,
        create_rendered_image_state(vec![chunk.clone()], vec![]),
    )]));
    output.add_pane_image_output_to_client(
        1,
        PaneImageRenderOutput {
            kitty_scene: crate::output::KittyRenderBundle {
                explicit_chunks: vec![chunk],
                ..Default::default()
            },
            kitty_host_state_cleared: true,
            ..Default::default()
        },
        None,
    );

    let serialized = output.serialize().unwrap();
    let client_output = serialized.get(&1).unwrap();
    assert!(
        client_output.contains("\u{1b}_Ga=d,d=A\u{1b}\\"),
        "a pane-local kitty clear must clear the host image namespace before rebuilding images"
    );
    assert!(
        client_output.contains("a=p"),
        "after a pane-local kitty clear the kitty scene should still be rebuilt"
    );
    assert!(
        client_output.contains("a=t"),
        "after a pane-local kitty clear assets should be retransmitted because host residency is gone"
    );
}

#[test]
fn test_kitty_diff_serialization_deletes_single_placement_without_delete_all() {
    let client_ids = create_test_clients(1);
    let mut first_chunk = create_kitty_chunk(1, 2, 2);
    first_chunk.placement_id = Some(pid(10));
    let mut second_chunk = create_kitty_chunk(2, 2, 2);
    second_chunk.placement_id = Some(pid(20));
    let removed_wire_pid = wire_pid_for_stable_render_id(second_chunk.stable_render_id);

    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.set_last_rendered_image_states(HashMap::from([(
        1,
        create_rendered_image_state(vec![first_chunk.clone(), second_chunk], vec![]),
    )]));
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
        client_output.contains(&format!(
            "\u{1b}_Ga=d,d=i,i=2,p={}\u{1b}\\",
            removed_wire_pid.wire_value()
        )),
        "single-placement delete should target only the removed placement"
    );
}

#[test]
fn test_kitty_diff_serialization_preserves_targeted_deletes_across_output_state_handoffs() {
    fn smoke_style_chunk(
        image_id: u32,
        cell_x: usize,
        cell_y: usize,
        z_index: i32,
    ) -> KittyImageChunk {
        let mut chunk = create_kitty_chunk(image_id, 12, 4);
        chunk.placement_id = Some(pid(1));
        chunk.cell_x = cell_x;
        chunk.cell_y = cell_y;
        chunk.z_index = z_index;
        chunk
    }

    let client_ids = create_test_clients(1);
    let top_left = smoke_style_chunk(201, 2, 8, -1);
    let top_right = smoke_style_chunk(202, 20, 8, 1);
    let bottom_left = smoke_style_chunk(203, 2, 16, 1);
    let bottom_right = smoke_style_chunk(204, 20, 16, -1);

    let frame1_scene = vec![
        top_left.clone(),
        top_right.clone(),
        bottom_left.clone(),
        bottom_right.clone(),
    ];
    let frame2_scene = vec![top_left.clone(), bottom_left.clone(), bottom_right.clone()];
    let frame3_scene = vec![
        top_left.clone(),
        top_right.clone(),
        bottom_left.clone(),
        bottom_right.clone(),
    ];
    let frame4_scene = vec![top_right.clone(), bottom_left, bottom_right];

    let (mut output1, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_test_output_with_state();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output1.add_clients(&client_ids, link_handler, None);
    output1.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(frame1_scene),
        None,
    );
    let _ = output1.serialize().unwrap();
    let state1 = output1.last_rendered_image_states();

    let (mut output2, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_test_output_with_state();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output2.add_clients(&client_ids, link_handler, None);
    output2.set_last_rendered_image_states(state1);
    output2.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(frame2_scene),
        None,
    );
    let frame2_serialized = output2.serialize().unwrap();
    let frame2_client_output = frame2_serialized.get(&1).unwrap();
    let top_right_wire_pid = wire_pid_for_stable_render_id(top_right.stable_render_id);
    assert!(
        frame2_client_output.contains(&format!(
            "\u{1b}_Ga=d,d=i,i=202,p={}\u{1b}\\",
            top_right_wire_pid.wire_value()
        )),
        "frame 2 should delete the removed top-right placement using its stable host placement id, got: {frame2_client_output:?}"
    );
    let state2 = output2.last_rendered_image_states();

    let (mut output3, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_test_output_with_state();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output3.add_clients(&client_ids, link_handler, None);
    output3.set_last_rendered_image_states(state2);
    output3.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(frame3_scene),
        None,
    );
    let _ = output3.serialize().unwrap();
    let state3 = output3.last_rendered_image_states();

    let (mut output4, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_test_output_with_state();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output4.add_clients(&client_ids, link_handler, None);
    output4.set_last_rendered_image_states(state3);
    output4.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(frame4_scene),
        None,
    );

    let frame4_serialized = output4.serialize().unwrap();
    let frame4_client_output = frame4_serialized.get(&1).unwrap();
    let top_left_wire_pid = wire_pid_for_stable_render_id(top_left.stable_render_id);
    assert!(
        !frame4_client_output.contains("\u{1b}_Ga=d,d=A\u{1b}\\"),
        "later targeted deletes should not fall back to delete-all after state handoff"
    );
    assert!(
        frame4_client_output.contains(&format!(
            "\u{1b}_Ga=d,d=i,i=201,p={}\u{1b}\\",
            top_left_wire_pid.wire_value()
        )),
        "frame 4 should still emit a targeted delete for the removed top-left placement using its stable host placement id, got: {frame4_client_output:?}"
    );
}

#[test]
fn test_kitty_diff_serialization_retransmits_asset_after_resident_retirement() {
    let client_ids = create_test_clients(1);
    let mut chunk = create_kitty_chunk(1, 2, 2);
    chunk.placement_id = Some(pid(10));

    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.set_last_rendered_image_states(HashMap::from([(
        1,
        create_rendered_image_state(vec![chunk.clone()], vec![]),
    )]));
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
        client_output.contains("a=t"),
        "re-placing a retired resident asset should retransmit image bytes"
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
    chunk.placement_id = Some(pid(10));

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
fn test_kitty_diff_serialization_no_longer_falls_back_for_unnamed_explicit_changes() {
    let client_ids = create_test_clients(1);
    let mut base_chunk = create_kitty_chunk(1, 2, 2);
    base_chunk.placement_id = None;
    let mut changed_chunk = base_chunk.clone();
    changed_chunk.columns = 3;

    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.set_last_rendered_image_states(HashMap::from([(
        1,
        create_rendered_image_state(vec![base_chunk], vec![]),
    )]));
    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(vec![changed_chunk]),
        None,
    );

    let serialized = output.serialize().unwrap();
    let client_output = serialized.get(&1).unwrap();

    assert!(
        !client_output.contains("\u{1b}_Ga=d,d=A\u{1b}\\"),
        "unnamed explicit placement changes should diff without delete-all fallback"
    );
    assert!(
        !client_output.contains("a=t") && client_output.contains("a=p"),
        "unnamed explicit placement changes should not retransmit resident assets"
    );
}

#[test]
fn test_kitty_diff_assigns_distinct_stable_synthesized_ids_across_modes() {
    let client_ids = create_test_clients(1);
    let mut chunk = create_kitty_chunk(1, 2, 2);
    chunk.placement_id = None;
    let mut render = create_kitty_placeholder_render(1);
    render.placement_id = None;

    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.add_pane_image_output_to_client(
        1,
        PaneImageRenderOutput {
            kitty_scene: crate::panes::pane_image_scene::KittyRenderBundle {
                explicit_chunks: vec![chunk],
                placeholder_renders: vec![render],
            },
            ..Default::default()
        },
        None,
    );

    let serialized = output.serialize().unwrap();
    let client_output = serialized.get(&1).unwrap();
    let explicit_wire_pid = wire_pid_for_stable_render_id(1);
    let placeholder_wire_pid = placeholder_wire_pid_for_stable_render_id(2);

    assert!(
        client_output.contains(&format!("a=p,i=1,p={}", explicit_wire_pid.wire_value())),
        "expected synthesized placement id for the explicit placement"
    );
    assert!(
        client_output.contains(&format!(
            "a=p,U=1,i=1,p={}",
            placeholder_wire_pid.wire_value()
        )),
        "expected the placeholder placement to receive a distinct synthesized id"
    );
}

#[test]
fn unnamed_kitty_placeholder_wire_id_fits_underline_color_encoding() {
    let client_ids = create_test_clients(1);
    let mut render = create_kitty_placeholder_render(200);
    render.placement_id = None;

    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.add_pane_image_output_to_client(
        1,
        PaneImageRenderOutput {
            kitty_scene: KittyRenderBundle {
                explicit_chunks: vec![],
                placeholder_renders: vec![render],
            },
            ..Default::default()
        },
        None,
    );

    let serialized = output.serialize().unwrap();
    let client_output = serialized.get(&1).unwrap();

    assert!(
        client_output.contains("a=p,U=1,i=200,p=400"),
        "placeholder virtual placement id must stay inside the 24-bit color-encoded namespace: {client_output:?}"
    );
    assert!(
        client_output.contains("\u{1b}[58;2;0;1;144m"),
        "placeholder cell underline color must encode the same placement id as p=400: {client_output:?}"
    );
}

#[test]
fn unnamed_kitty_explicit_changes_should_diff_without_full_reset() {
    let client_ids = create_test_clients(1);
    let mut base_chunk = create_kitty_chunk(1, 2, 2);
    base_chunk.placement_id = None;
    let mut changed_chunk = base_chunk.clone();
    changed_chunk.columns = 3;

    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.set_last_rendered_image_states(HashMap::from([(
        1,
        create_rendered_image_state(vec![base_chunk], vec![]),
    )]));
    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(vec![changed_chunk]),
        None,
    );

    let serialized = output.serialize().unwrap();
    let client_output = serialized.get(&1).unwrap();

    assert!(
        !client_output.contains("\u{1b}_Ga=d,d=A\u{1b}\\"),
        "unnamed explicit placement changes should not force delete-all fallback"
    );
    assert!(
        !client_output.contains("a=t"),
        "unnamed explicit placement changes should diff in-place without retransmitting assets"
    );
    assert!(
        client_output.contains("a=p"),
        "unnamed explicit placement changes should still emit placement traffic"
    );
}

#[test]
fn unnamed_kitty_placeholder_changes_should_diff_without_full_reset() {
    let client_ids = create_test_clients(1);
    let mut base_render = create_kitty_placeholder_render(1);
    base_render.placement_id = None;
    let mut changed_render = base_render.clone();
    changed_render.columns = 3;
    changed_render.cells = vec![
        KittyPlaceholderCellRender {
            cell_x: 0,
            cell_y: 0,
            placeholder_row: 0,
            placeholder_col: 0,
        },
        KittyPlaceholderCellRender {
            cell_x: 1,
            cell_y: 0,
            placeholder_row: 0,
            placeholder_col: 1,
        },
        KittyPlaceholderCellRender {
            cell_x: 2,
            cell_y: 0,
            placeholder_row: 0,
            placeholder_col: 2,
        },
    ];

    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.set_last_rendered_image_states(HashMap::from([(
        1,
        create_rendered_image_state(vec![], vec![base_render]),
    )]));
    output.add_pane_image_output_to_client(
        1,
        PaneImageRenderOutput {
            kitty_scene: KittyRenderBundle {
                explicit_chunks: vec![],
                placeholder_renders: vec![changed_render],
            },
            ..Default::default()
        },
        None,
    );

    let serialized = output.serialize().unwrap();
    let client_output = serialized.get(&1).unwrap();

    assert!(
        !client_output.contains("\u{1b}_Ga=d,d=A\u{1b}\\"),
        "unnamed placeholder changes should not force delete-all fallback"
    );
    assert!(
        !client_output.contains("a=t"),
        "unnamed placeholder changes should diff in-place without retransmitting assets"
    );
    assert!(
        client_output.contains("U=1"),
        "unnamed placeholder changes should still emit placeholder placement traffic"
    );
}

#[test]
fn test_prepared_image_output_emits_kitty_delete_before_text_when_scene_changes() {
    let client_ids = create_test_clients(1);
    let mut base_chunk = create_kitty_chunk(77, 2, 2);
    base_chunk.placement_id = Some(pid(10));
    let base_wire_pid = wire_pid_for_stable_render_id(base_chunk.stable_render_id);
    let mut changed_chunk = base_chunk.clone();
    changed_chunk.columns = 3;
    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.set_last_rendered_image_states(HashMap::from([(
        1,
        create_rendered_image_state(vec![base_chunk], vec![]),
    )]));

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
    let delete_pos = client_output
        .find(&format!("a=d,d=i,i=77,p={}", base_wire_pid.wire_value()))
        .unwrap();
    let text_pos = client_output.find("TEXT-PHASE").unwrap();

    assert!(
        delete_pos < text_pos,
        "kitty placement deletes should happen before text when the scene changes"
    );
}

#[test]
fn test_output_round_trips_last_rendered_image_state_for_single_client() {
    let mut output = create_test_output();
    let mut explicit_chunk = create_kitty_chunk(9, 2, 2);
    explicit_chunk.placement_id = Some(pid(19));
    let mut placeholder_render = create_kitty_placeholder_render(9);
    placeholder_render.placement_id = Some(pid(29));
    let expected_state = RenderedImageState {
        explicit_chunks: vec![explicit_chunk.clone()],
        placeholder_renders: vec![placeholder_render.clone()],
        resident_asset_generations: HashMap::from([(9, 42)]),
    };
    let mut expected_scene = KittySceneState::default();
    expected_scene.insert_asset(9, 42);
    let expected_snapshot = Rc::new(LastRenderedImageState::with_kitty_scene_state(
        expected_state.clone(),
        Some(expected_scene),
    ));

    output.set_last_rendered_image_state_for_client(7, Rc::clone(&expected_snapshot));
    let actual_snapshot = output
        .last_rendered_image_state_for_client(7)
        .expect("client image state should round-trip");
    assert!(
        Rc::ptr_eq(&actual_snapshot, &expected_snapshot),
        "previous render state should be shared by snapshot reference"
    );
    let actual_state = actual_snapshot.rendered_image_state();

    assert_eq!(actual_state.explicit_chunks, expected_state.explicit_chunks);
    assert_eq!(
        actual_state.placeholder_renders,
        expected_state.placeholder_renders
    );
    assert_eq!(
        actual_state.resident_asset_generations,
        expected_state.resident_asset_generations
    );
    assert!(output.last_rendered_image_state_for_client(8).is_none());
}

#[test]
fn test_output_round_trips_cached_kitty_scene_snapshot() {
    let (mut output, _sixel_image_store, kitty_asset_store, _character_cell_size) =
        create_test_output_with_state();
    let chunk = create_kitty_chunk(9, 2, 2);
    kitty_asset_store.borrow_mut().insert_asset(
        chunk.image_id,
        KittyImageData::Png {
            data: vec![1],
            width: 1,
            height: 1,
        },
    );
    output.add_pane_image_output_to_client(
        7,
        pane_image_output_with_kitty_scene(vec![chunk.clone()]),
        None,
    );

    output.serialize().unwrap();
    let snapshot = output
        .last_rendered_image_state_for_client(7)
        .expect("serialized kitty output should publish last rendered image state");
    assert!(
        snapshot.kitty_scene_state().is_some(),
        "snapshot should carry the cached scene state"
    );
    assert!(
        snapshot
            .rendered_image_state()
            .resident_asset_generations
            .is_empty(),
        "cached scene snapshots should not duplicate resident asset state"
    );
    assert_eq!(
        snapshot.resident_asset_generations().get(&chunk.image_id),
        kitty_asset_store
            .borrow()
            .generation(chunk.image_id)
            .as_ref()
    );

    let mut next_output = create_test_output();
    next_output.set_last_rendered_image_state_for_client(7, Rc::clone(&snapshot));
    let next_snapshot = next_output
        .last_rendered_image_state_for_client(7)
        .expect("snapshot should be installed by reference");
    assert!(Rc::ptr_eq(&snapshot, &next_snapshot));
}

#[test]
fn test_output_drops_resident_assets_that_are_no_longer_local() {
    let client_ids = create_test_clients(1);
    let (mut output, _sixel_image_store, kitty_asset_store, _character_cell_size) =
        create_test_output_with_state();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.set_last_rendered_image_states(HashMap::from([(
        1,
        RenderedImageState {
            resident_asset_generations: HashMap::from([(250, 1)]),
            ..Default::default()
        },
    )]));
    kitty_asset_store.borrow_mut().remove_asset(250);
    output
        .add_character_chunks_to_client(
            1,
            vec![create_character_chunk_from_str("TEXT-ONLY", 0, 0)],
            None,
        )
        .unwrap();

    output.serialize().unwrap();

    assert!(
        output.last_rendered_image_state_for_client(1).is_none(),
        "resident asset generations without matching local assets should not persist"
    );
}

#[test]
fn test_output_drops_resident_assets_that_are_no_longer_desired() {
    let client_ids = create_test_clients(1);
    let (mut output, _sixel_image_store, _kitty_asset_store, _character_cell_size) =
        create_test_output_with_state();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    output.add_clients(&client_ids, link_handler, None);
    output.set_last_rendered_image_states(HashMap::from([(
        1,
        RenderedImageState {
            resident_asset_generations: HashMap::from([(250, 1)]),
            ..Default::default()
        },
    )]));
    output
        .add_character_chunks_to_client(
            1,
            vec![create_character_chunk_from_str("TEXT-ONLY", 0, 0)],
            None,
        )
        .unwrap();

    output.serialize().unwrap();

    assert!(
        output.last_rendered_image_state_for_client(1).is_none(),
        "resident asset generations with no desired placements should not persist"
    );
}

#[test]
fn test_prepared_image_output_serializes_sixels_after_text() {
    let client_ids = create_test_clients(1);
    let (mut output, sixel_image_store, _kitty_asset_store, character_cell_size) =
        create_test_output_with_state();
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
    let (mut output, _sixel_image_store, kitty_asset_store, _character_cell_size) =
        create_test_output_with_state();
    seed_test_kitty_asset(kitty_asset_store, 88, create_kitty_image_data(88));
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
fn real_explicit_placement_uses_stable_render_id_as_host_id() {
    let client_ids = create_test_clients(1);
    let mut chunk = create_kitty_chunk(203, 2, 2);
    chunk.stable_render_id = 405;

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

    assert!(
        client_output.contains("a=p,i=203,p=405"),
        "real explicit host placement id should be the allocated stable render id, got: {client_output:?}"
    );
    assert!(
        !client_output.contains("a=p,i=203,p=2147484053"),
        "real explicit host placement id should not be remapped away from the allocated id"
    );
}

#[test]
fn test_prepare_render_body_derives_sixel_fragments() {
    let client_ids = create_test_clients(1);
    let (mut output, sixel_image_store, _kitty_asset_store, character_cell_size) =
        create_test_output_with_state();
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
        ImageFragment::Sixel(chunk) => {
            assert_eq!(chunk.sixel_image_id, 1);
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
    match prepared.after_text.kitty_plan {
        KittyScenePlan::Diff {
            asset_ops,
            placement_ops,
        } => {
            assert_eq!(
                asset_ops,
                vec![KittyAssetOp::EnsureResident {
                    image_id: 91,
                    generation: 1,
                }]
            );
            assert_eq!(placement_ops.len(), 1);
            match &placement_ops[0] {
                KittyPlacementOp::PlaceExplicit { key, chunk } => {
                    assert_eq!(key.image_id, 91);
                    assert_eq!(
                        key.wire_placement_id,
                        wire_pid_for_stable_render_id(chunk.stable_render_id)
                    );
                    assert_eq!(chunk.placement_id, Some(PlacementId::Protocol(91)));
                    assert_eq!(*chunk, create_kitty_chunk(91, 2, 2));
                },
                other => panic!("expected explicit placement op, got {other:?}"),
            }
        },
        other => panic!("expected diff kitty plan, got {other:?}"),
    }
}

#[test]
fn test_prepare_render_body_serializes_multi_occluder_kitty_explicit_fragments() {
    let client_ids = create_test_clients(1);
    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    let floating_panes_stack = FloatingPanesStack {
        layers: vec![create_pane_geom(2, 2, 4, 4), create_pane_geom(5, 0, 2, 5)],
    };
    output.add_clients(&client_ids, link_handler, Some(floating_panes_stack));

    let chunk = KittyImageChunk {
        stable_render_id: 97,
        image_id: 97,
        placement_id: Some(pid(97)),
        cell_x: 0,
        cell_y: 0,
        columns: 8,
        rows: 8,
        columns_specified: true,
        rows_specified: true,
        source_x: 10,
        source_y: 20,
        source_width: 240,
        source_height: 160,
        z_index: 0,
        x_offset: 0,
        y_offset: 0,
    };
    let occluders = [
        TestRect {
            x: 2,
            y: 2,
            columns: 4,
            rows: 4,
        },
        TestRect {
            x: 5,
            y: 0,
            columns: 2,
            rows: 5,
        },
    ];
    let expected = expected_explicit_fragments_for_occluders(&chunk, &occluders);

    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(vec![chunk]),
        Some(0),
    );

    let prepared = output.image_output.prepare_render_body_for_client(1, false);
    match &prepared.after_text.kitty_plan {
        KittyScenePlan::Diff {
            asset_ops,
            placement_ops,
        } => {
            assert_eq!(asset_ops.len(), 1, "expected one resident-asset op");
            let actual = placement_ops
                .iter()
                .map(|placement_op| match placement_op {
                    KittyPlacementOp::PlaceExplicit { chunk, .. } => chunk.clone(),
                    other => panic!("expected only explicit placement ops, got {other:?}"),
                })
                .collect::<Vec<_>>();
            assert_kitty_chunk_sets_eq(&actual, &expected);
        },
        other => panic!("expected diff kitty plan, got {other:?}"),
    }

    let mut serialized = String::new();
    prepared
        .after_text
        .serialize(&mut output.image_output, None, &mut serialized)
        .unwrap();
    assert_eq!(
        serialized.matches("\u{1b}_Ga=p,").count(),
        expected.len(),
        "serialized output should place every surviving explicit fragment",
    );
    assert_eq!(
        serialized.matches("\u{1b}_Ga=t,").count(),
        1,
        "serialized output should ensure the asset once for all fragments",
    );
}

#[test]
fn test_prepare_render_body_serializes_multi_occluder_kitty_explicit_fragments_with_columns_only() {
    let client_ids = create_test_clients(1);
    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    let floating_panes_stack = FloatingPanesStack {
        layers: vec![create_pane_geom(2, 2, 4, 4), create_pane_geom(5, 0, 2, 5)],
    };
    output.add_clients(&client_ids, link_handler, Some(floating_panes_stack));

    let expected_geometry = KittyImageChunk {
        stable_render_id: 98,
        image_id: 98,
        placement_id: Some(pid(98)),
        cell_x: 0,
        cell_y: 0,
        columns: 8,
        rows: 8,
        columns_specified: true,
        rows_specified: false,
        source_x: 10,
        source_y: 20,
        source_width: 240,
        source_height: 160,
        z_index: 0,
        x_offset: 0,
        y_offset: 0,
    };
    let occluders = [
        TestRect {
            x: 2,
            y: 2,
            columns: 4,
            rows: 4,
        },
        TestRect {
            x: 5,
            y: 0,
            columns: 2,
            rows: 5,
        },
    ];
    let mut expected = expected_explicit_fragments_for_occluders(&expected_geometry, &occluders);
    for fragment in &mut expected {
        fragment.columns_specified = true;
        fragment.rows_specified = true;
    }

    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(vec![expected_geometry.clone()]),
        Some(0),
    );

    let prepared = output.image_output.prepare_render_body_for_client(1, false);
    match &prepared.after_text.kitty_plan {
        KittyScenePlan::Diff {
            asset_ops,
            placement_ops,
        } => {
            assert_eq!(asset_ops.len(), 1, "expected one resident-asset op");
            let actual = placement_ops
                .iter()
                .map(|placement_op| match placement_op {
                    KittyPlacementOp::PlaceExplicit { chunk, .. } => chunk.clone(),
                    other => panic!("expected only explicit placement ops, got {other:?}"),
                })
                .collect::<Vec<_>>();
            assert_kitty_chunk_sets_eq(&actual, &expected);
        },
        other => panic!("expected diff kitty plan, got {other:?}"),
    }

    let mut serialized = String::new();
    prepared
        .after_text
        .serialize(&mut output.image_output, None, &mut serialized)
        .unwrap();
    assert_eq!(
        serialized.matches("\u{1b}_Ga=p,").count(),
        expected.len(),
        "serialized output should place every surviving explicit fragment",
    );
    assert_eq!(
        serialized.matches("\u{1b}_Ga=t,").count(),
        1,
        "serialized output should ensure the asset once for all fragments",
    );
    assert!(
        serialized.contains(",r="),
        "split columns-only fragments should serialize bounded r= to preserve the original resolved transform",
    );
}

#[test]
fn test_prepare_render_body_serializes_multi_occluder_kitty_explicit_fragments_with_rows_only() {
    let client_ids = create_test_clients(1);
    let mut output = create_test_output();
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    let floating_panes_stack = FloatingPanesStack {
        layers: vec![create_pane_geom(2, 2, 4, 4), create_pane_geom(5, 0, 2, 5)],
    };
    output.add_clients(&client_ids, link_handler, Some(floating_panes_stack));

    let expected_geometry = KittyImageChunk {
        stable_render_id: 99,
        image_id: 99,
        placement_id: Some(pid(99)),
        cell_x: 0,
        cell_y: 0,
        columns: 8,
        rows: 8,
        columns_specified: false,
        rows_specified: true,
        source_x: 10,
        source_y: 20,
        source_width: 240,
        source_height: 160,
        z_index: 0,
        x_offset: 0,
        y_offset: 0,
    };
    let occluders = [
        TestRect {
            x: 2,
            y: 2,
            columns: 4,
            rows: 4,
        },
        TestRect {
            x: 5,
            y: 0,
            columns: 2,
            rows: 5,
        },
    ];
    let mut expected = expected_explicit_fragments_for_occluders(&expected_geometry, &occluders);
    for fragment in &mut expected {
        fragment.columns_specified = true;
        fragment.rows_specified = true;
    }

    output.add_pane_image_output_to_client(
        1,
        pane_image_output_with_kitty_scene(vec![expected_geometry.clone()]),
        Some(0),
    );

    let prepared = output.image_output.prepare_render_body_for_client(1, false);
    match &prepared.after_text.kitty_plan {
        KittyScenePlan::Diff {
            asset_ops,
            placement_ops,
        } => {
            assert_eq!(asset_ops.len(), 1, "expected one resident-asset op");
            let actual = placement_ops
                .iter()
                .map(|placement_op| match placement_op {
                    KittyPlacementOp::PlaceExplicit { chunk, .. } => chunk.clone(),
                    other => panic!("expected only explicit placement ops, got {other:?}"),
                })
                .collect::<Vec<_>>();
            assert_kitty_chunk_sets_eq(&actual, &expected);
        },
        other => panic!("expected diff kitty plan, got {other:?}"),
    }

    let mut serialized = String::new();
    prepared
        .after_text
        .serialize(&mut output.image_output, None, &mut serialized)
        .unwrap();
    assert_eq!(
        serialized.matches("\u{1b}_Ga=p,").count(),
        expected.len(),
        "serialized output should place every surviving explicit fragment",
    );
    assert_eq!(
        serialized.matches("\u{1b}_Ga=t,").count(),
        1,
        "serialized output should ensure the asset once for all fragments",
    );
    assert!(
        serialized.contains(",c="),
        "split rows-only fragments should serialize bounded c= to preserve the original resolved transform",
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
                generation: 1,
            }],
            placement_ops: vec![KittyPlacementOp::PlacePlaceholder {
                key: KittyPlacementKey {
                    stable_render_id: 184,
                    image_id: 92,
                    wire_placement_id: placeholder_wire_pid_for_stable_render_id(184),
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
        vec![ImageFragment::KittyExplicit(chunk)],
        Some(0),
        None,
    );

    assert_eq!(fragments.len(), 1);
    match &fragments[0] {
        ImageFragment::KittyExplicit(chunk) => {
            assert_eq!(chunk.cell_x, 0);
            assert_eq!(chunk.columns, 1);
        },
        other => panic!("expected kitty explicit fragment, got {other:?}"),
    }
}

#[test]
fn test_clip_kitty_explicit_fragment_assigns_distinct_split_placement_id() {
    let stack = FloatingPanesStack {
        layers: vec![create_pane_geom(1, 0, 1, 2)],
    };
    let chunk = KittyImageChunk {
        cell_x: 0,
        cell_y: 0,
        columns: 2,
        rows: 2,
        placement_id: Some(pid(777)),
        ..create_kitty_chunk(194, 2, 2)
    };
    let fragments = visible_image_fragments(
        &stack,
        vec![ImageFragment::KittyExplicit(chunk)],
        Some(0),
        None,
    );

    assert_eq!(fragments.len(), 1);
    match &fragments[0] {
        ImageFragment::KittyExplicit(chunk) => {
            let Some(PlacementId::Synthetic(fragment_placement_id)) = chunk.placement_id else {
                panic!("expected split fragment to receive a synthetic placement id");
            };
            assert!(
                fragment_placement_id
                    >= crate::output::kitty_host_placement_id::EXPLICIT_FRAGMENT_PLACEMENT_ID_MIN,
                "split fragment placement id should be in the explicit-only range"
            );
            assert_ne!(
                fragment_placement_id, chunk.stable_render_id as u32,
                "split fragment placement id should not collide with the base real placement id"
            );
        },
        other => panic!("expected kitty explicit fragment, got {other:?}"),
    }
}

#[test]
fn test_clip_kitty_explicit_fragment_against_two_covering_panes() {
    let stack = FloatingPanesStack {
        layers: vec![create_pane_geom(2, 2, 4, 4), create_pane_geom(5, 0, 2, 5)],
    };
    let chunk = KittyImageChunk {
        cell_x: 0,
        cell_y: 0,
        columns: 8,
        rows: 8,
        source_width: 80,
        source_height: 80,
        ..create_kitty_chunk(95, 8, 8)
    };
    let occluders = [
        TestRect {
            x: 2,
            y: 2,
            columns: 4,
            rows: 4,
        },
        TestRect {
            x: 5,
            y: 0,
            columns: 2,
            rows: 5,
        },
    ];
    let expected = expected_explicit_fragments_for_occluders(&chunk, &occluders);

    let fragments = visible_image_fragments(
        &stack,
        vec![ImageFragment::KittyExplicit(chunk)],
        Some(0),
        None,
    );

    let actual = fragments
        .into_iter()
        .map(|fragment| match fragment {
            ImageFragment::KittyExplicit(chunk) => chunk,
            other => panic!("expected kitty explicit fragment, got {other:?}"),
        })
        .collect::<Vec<_>>();
    assert_kitty_chunk_sets_eq(&actual, &expected);
}

#[test]
fn test_clip_cropped_kitty_explicit_fragment_against_two_covering_panes() {
    let stack = FloatingPanesStack {
        layers: vec![create_pane_geom(2, 2, 4, 4), create_pane_geom(5, 0, 2, 5)],
    };
    let chunk = KittyImageChunk {
        cell_x: 0,
        cell_y: 0,
        columns: 8,
        rows: 8,
        source_x: 100,
        source_y: 200,
        source_width: 240,
        source_height: 160,
        ..create_kitty_chunk(96, 8, 8)
    };
    let occluders = [
        TestRect {
            x: 2,
            y: 2,
            columns: 4,
            rows: 4,
        },
        TestRect {
            x: 5,
            y: 0,
            columns: 2,
            rows: 5,
        },
    ];
    let expected = expected_explicit_fragments_for_occluders(&chunk, &occluders);

    let fragments = visible_image_fragments(
        &stack,
        vec![ImageFragment::KittyExplicit(chunk)],
        Some(0),
        None,
    );

    let actual = fragments
        .into_iter()
        .map(|fragment| match fragment {
            ImageFragment::KittyExplicit(chunk) => chunk,
            other => panic!("expected kitty explicit fragment, got {other:?}"),
        })
        .collect::<Vec<_>>();
    assert_kitty_chunk_sets_eq(&actual, &expected);
}

#[test]
fn test_clip_sixel_fragment_against_covering_pane() {
    let stack = FloatingPanesStack {
        layers: vec![create_pane_geom(0, 0, 3, 3)],
    };
    let fragments = visible_image_fragments(
        &stack,
        vec![ImageFragment::Sixel(SixelImageChunk {
            cell_x: 0,
            cell_y: 0,
            sixel_image_pixel_x: 0,
            sixel_image_pixel_y: 0,
            sixel_image_pixel_width: 20,
            sixel_image_pixel_height: 40,
            sixel_image_id: 1,
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
        vec![ImageFragment::KittyPlaceholder(render)],
        Some(0),
        None,
    );

    assert_eq!(fragments.len(), 1);
    match &fragments[0] {
        ImageFragment::KittyPlaceholder(render) => {
            assert_eq!(render.cells.len(), 1);
            assert_eq!(render.cells[0].cell_x, 1);
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
fn test_multi_client_image_output_registers_image_only_clients() {
    let mut output = create_test_output();
    let client_ids = create_test_clients(2);

    output.add_pane_image_output_to_multiple_clients(
        pane_image_output_with_kitty_scene(vec![create_kitty_chunk(1, 2, 2)]),
        client_ids.iter().copied(),
        None,
    );

    let result = output.serialize().unwrap();
    assert_eq!(
        result.len(),
        2,
        "image-only multi-client output should serialize every target client"
    );
    for client_id in client_ids {
        let client_output = result
            .get(&client_id)
            .unwrap_or_else(|| panic!("image-only multi-client output missing client {client_id}"));
        assert!(
            client_output.contains("\u{1b}_G"),
            "client {client_id} should receive kitty image output"
        );
    }
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
