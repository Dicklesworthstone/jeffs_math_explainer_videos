//! The ordering-trace corpus (fm-jsc acceptance, §8.5): the tricky cases of
//! the draw-order model, each one a ruling in `docs/RENDER_ORDER.md` with a
//! citation into the pinned Reference.
//!
//! Traces compare *identity and batch structure*, never geometry — this is
//! the order suite, and a trace that also hashed points would fail for
//! reasons that have nothing to do with order.

use fmn_mobject::order::{PassOrder, ProgramKind};
use fmn_mobject::{JointType, Mob, Mobject, RenderPrimitive, Stage, Uniforms};

/// A pointful vector mobject (one record) — only pointful, paint-bearing
/// family members draw.
fn dot(stage: &mut Stage) -> Mob {
    stage.add(Mobject::vector_from_points(&[[0.0, 0.0, 0.0]]))
}

/// A pointful mobject with an explicit uniform inventory.
fn styled(stage: &mut Stage, uniforms: Uniforms) -> Mob {
    let mob = dot(stage);
    if let Some(entry) = stage.get_mut(mob) {
        *entry.uniforms_mut() = uniforms;
    }
    mob
}

/// A pointless container (a `VGroup`): it never draws, its children do.
fn group(stage: &mut Stage, children: &[Mob]) -> Mob {
    let parent = stage.add(Mobject::new());
    for &child in children {
        stage.attach(parent, child).expect("attach");
    }
    parent
}

// -------------------------------------------------------- the scene list

#[test]
fn the_scene_list_is_the_draw_order_back_to_front() {
    let mut stage = Stage::new();
    let a = dot(&mut stage);
    let b = dot(&mut stage);
    let c = dot(&mut stage);
    for m in [a, b, c] {
        stage.add_to_scene(m).expect("root");
    }
    assert_eq!(stage.draw_plan().sequence(), [a, b, c]);
}

/// A deterministic scene of shared-child DAG families: `leaves` pointful
/// members and `groups` containers, each over 1-3 earlier members, with a
/// few of them rooted. `seed` drives a 64-bit LCG.
fn random_dag(stage: &mut Stage, seed: u64, leaves: usize, groups: usize) -> Vec<Mob> {
    let mut state = seed;
    let mut next = |bound: usize| {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        usize::try_from(state >> 33).expect("31 bits") % bound
    };
    let mut members: Vec<Mob> = (0..leaves).map(|_| dot(stage)).collect();
    for _ in 0..groups {
        let children: Vec<Mob> = (0..=next(3))
            .map(|_| members[next(members.len())])
            .collect();
        let mut unique = Vec::new();
        for child in children {
            if !unique.contains(&child) {
                unique.push(child);
            }
        }
        members.push(group(stage, &unique));
    }
    for _ in 0..4 {
        stage
            .add_to_scene(members[next(members.len())])
            .expect("root");
    }
    members
}

/// The previous `Stage::family`: depth-first, each member once, found by a
/// linear search over the output.
fn family_oracle(stage: &Stage, mob: Mob) -> Vec<Mob> {
    let mut out = Vec::new();
    let mut stack = vec![mob];
    while let Some(current) = stack.pop() {
        if out.contains(&current) {
            continue;
        }
        if let Some(entry) = stage.get(current) {
            out.push(current);
            stack.extend(entry.submobjects().iter().rev());
        }
    }
    out
}

/// The previous `recursive_remove`: descends into every member.
fn remove_oracle(stage: &Stage, members: &[Mob], targets: &[Mob]) -> (Vec<Mob>, bool) {
    let mut kept = Vec::new();
    let mut found = false;
    for &member in members {
        if targets.contains(&member) {
            found = true;
            continue;
        }
        let children = stage
            .get(member)
            .map(|entry| entry.submobjects().to_vec())
            .unwrap_or_default();
        let (sub, below) = remove_oracle(stage, &children, targets);
        if below {
            kept.extend(sub);
            found = true;
        } else {
            kept.push(member);
        }
    }
    (kept, found)
}

#[test]
fn family_and_removal_match_the_linear_search_walks() {
    // fm-5wq.31: `family` deduplicates through a set, and removal descends
    // only into the removed members' ancestors. Both must reproduce the
    // previous walks exactly, diamonds and shared children included.
    let mut shared = 0;
    for seed in 1..=60_u64 {
        let mut stage = Stage::new();
        let members = random_dag(&mut stage, seed, 8, 14);
        for &mob in &members {
            assert_eq!(stage.family(mob), family_oracle(&stage, mob), "seed {seed}");
        }
        shared += usize::from(
            members
                .iter()
                .any(|&m| stage.get(m).is_some_and(|e| e.parents().len() > 1)),
        );
        let removed = [
            members[(seed as usize * 7) % members.len()],
            members[seed as usize % members.len()],
        ];
        let mut targets = Vec::new();
        for &mob in &removed {
            for member in family_oracle(&stage, mob) {
                if !targets.contains(&member) {
                    targets.push(member);
                }
            }
        }
        let (expected, _) = remove_oracle(&stage, stage.roots(), &targets);
        stage.remove_many_from_scene(&removed);
        assert_eq!(stage.roots(), expected.as_slice(), "seed {seed}");
    }
    assert!(shared > 0, "no generated scene shared a child");
}

#[test]
fn the_topology_epoch_moves_with_every_draw_list_or_edge_change() {
    // fm-5wq.31: the Python binding reuses its projection of the scene while
    // this epoch holds, so every change a root-family walk can see must move
    // it. Detached copies and pins are invisible to that walk.
    let mut stage = Stage::new();
    let a = dot(&mut stage);
    let b = dot(&mut stage);
    let g = group(&mut stage, &[a]);
    let mut last = stage.topology_epoch();
    let mut moved = |stage: &Stage, what: &str| {
        let now = stage.topology_epoch();
        assert_ne!(now, last, "{what} left the topology epoch unchanged");
        last = now;
    };
    stage.add_to_scene(g).expect("root");
    moved(&stage, "add_to_scene");
    stage.attach(g, b).expect("attach");
    moved(&stage, "attach");
    stage.detach(g, b);
    moved(&stage, "detach");
    stage.add_to_scene(b).expect("root");
    moved(&stage, "a second add_to_scene");
    stage.bring_to_back(b).expect("back");
    moved(&stage, "bring_to_back");
    let c = dot(&mut stage);
    stage.replace_in_scene(b, &[c]).expect("replace");
    moved(&stage, "replace_in_scene");
    let snapshot = stage.snapshot();
    stage.remove_from_scene(c);
    moved(&stage, "remove_from_scene");
    stage.restore(&snapshot);
    moved(&stage, "restore");
    stage.delete(a).expect("delete");
    moved(&stage, "delete");

    let still = stage.topology_epoch();
    stage.copy_family(g).expect("detached copy");
    stage.pin(g).expect("pin");
    stage.unpin(g);
    assert_eq!(stage.topology_epoch(), still);
    assert_ne!(Stage::new().topology_epoch(), still);
}

/// Every member's children and parents, and the draw list, as indices into
/// `members`, so two stages built from one seed compare structurally.
fn graph_shape(stage: &Stage, members: &[Mob]) -> Vec<Vec<usize>> {
    let index = |mob: &Mob| members.iter().position(|m| m == mob).expect("known member");
    let mut shape: Vec<Vec<usize>> = Vec::new();
    for &mob in members {
        let entry = stage.get(mob).expect("live");
        shape.push(entry.submobjects().iter().map(index).collect());
        shape.push(entry.parents().iter().map(index).collect());
    }
    shape.push(stage.roots().iter().map(index).collect());
    shape
}

#[test]
fn replace_children_leaves_the_graph_detach_then_attach_leaves() {
    // fm-5wq.31: `replace_children` must end where detaching every child and
    // attaching each new one ends, parents order included, and a refusal
    // must change nothing.
    let mut refused = 0;
    for seed in 1..=60_u64 {
        let mut ours = Stage::new();
        let mut theirs = Stage::new();
        let a = random_dag(&mut ours, seed, 8, 14);
        let b = random_dag(&mut theirs, seed, 8, 14);
        let pick = |salt: usize| (seed as usize).wrapping_mul(31).wrapping_add(salt * 17) % a.len();
        let parent = pick(1);
        // Current children reordered and thinned, plus new ones and a repeat.
        let mut chosen: Vec<usize> = ours
            .get(a[parent])
            .expect("live")
            .submobjects()
            .iter()
            .rev()
            .map(|mob| a.iter().position(|m| m == mob).expect("known"))
            .skip(seed as usize % 2)
            .collect();
        chosen.extend([pick(2), pick(3), pick(2)]);
        let before = graph_shape(&ours, &a);
        let result =
            ours.replace_children(a[parent], &chosen.iter().map(|&i| a[i]).collect::<Vec<_>>());
        if result.is_err() {
            refused += 1;
            assert_eq!(
                graph_shape(&ours, &a),
                before,
                "seed {seed}: a refusal changed the graph"
            );
            continue;
        }
        let current = theirs.get(b[parent]).expect("live").submobjects().to_vec();
        for child in current {
            theirs.detach(b[parent], child);
        }
        for &i in &chosen {
            theirs.attach(b[parent], b[i]).expect("ours accepted it");
        }
        assert_eq!(
            graph_shape(&ours, &a),
            graph_shape(&theirs, &b),
            "seed {seed}"
        );
    }
    assert!(refused < 60, "every generated replacement was refused");
}

#[test]
fn an_entry_children_epoch_moves_exactly_with_its_child_list() {
    // fm-5wq.31: the binding re-verifies only members whose child list
    // moved, so each edit must move the edited parent's epoch, and nothing
    // else need move a bystander's.
    let mut stage = Stage::new();
    let a = dot(&mut stage);
    let b = dot(&mut stage);
    let g = group(&mut stage, &[a]);
    let h = group(&mut stage, &[b]);
    stage.add_to_scene(g).expect("root");
    stage.add_to_scene(h).expect("root");
    let epoch = |stage: &Stage, mob: Mob| stage.get(mob).expect("live").children_epoch();
    let (g0, h0) = (epoch(&stage, g), epoch(&stage, h));
    stage.attach(g, b).expect("attach");
    let g1 = epoch(&stage, g);
    assert_ne!(g1, g0, "attach");
    assert_eq!(epoch(&stage, h), h0, "a bystander moved on attach");
    stage.attach(g, b).expect("repeat attach is a no-op");
    assert_eq!(epoch(&stage, g), g1, "a no-op attach moved the epoch");
    stage.detach(g, b);
    let g2 = epoch(&stage, g);
    assert_ne!(g2, g1, "detach");
    stage.bring_to_back(h).expect("back");
    stage.pin(g).expect("pin");
    stage.unpin(g);
    assert_eq!(
        (epoch(&stage, g), epoch(&stage, h)),
        (g2, h0),
        "draw-list edits"
    );
    let copy = stage.copy_family(g).expect("copy");
    assert_ne!(epoch(&stage, copy), g2, "a copy shares its source's epoch");
    let snapshot = stage.snapshot();
    stage.restore(&snapshot);
    assert_ne!(epoch(&stage, g), g2, "restore kept a pre-restore epoch");
    let g3 = epoch(&stage, g);
    stage.delete(a).expect("delete");
    assert_ne!(epoch(&stage, g), g3, "deleting a child");
}

#[test]
fn equal_z_index_keeps_insertion_order() {
    // R-6: the sort key is (z_index, position), so it is stable.
    let mut stage = Stage::new();
    let mobs: Vec<Mob> = (0..5).map(|_| dot(&mut stage)).collect();
    for &m in &mobs {
        stage.add_to_scene(m).expect("root");
    }
    assert_eq!(stage.draw_plan().sequence(), mobs);
}

#[test]
fn z_index_sorts_the_scene_stably_on_add() {
    // R-6 with mixed keys: higher z_index draws later; ties keep the order
    // they were added in.
    let mut stage = Stage::new();
    let back = dot(&mut stage);
    let front = dot(&mut stage);
    let middle_a = dot(&mut stage);
    let middle_b = dot(&mut stage);
    stage.set_z_index(back, -5, true);
    stage.set_z_index(front, 10, true);
    // Added front-first, deliberately: the sort must reorder them.
    for m in [front, middle_a, back, middle_b] {
        stage.add_to_scene(m).expect("root");
    }
    assert_eq!(
        stage.draw_plan().sequence(),
        [back, middle_a, middle_b, front]
    );
}

#[test]
fn re_adding_a_member_promotes_it() {
    // R-7 / `bring_to_front` is `add` (scene.py:389).
    let mut stage = Stage::new();
    let a = dot(&mut stage);
    let b = dot(&mut stage);
    let c = dot(&mut stage);
    for m in [a, b, c] {
        stage.add_to_scene(m).expect("root");
    }
    stage.add_to_scene(a).expect("re-add");
    assert_eq!(stage.draw_plan().sequence(), [b, c, a]);
    stage.bring_to_front(b).expect("promote");
    assert_eq!(stage.draw_plan().sequence(), [c, a, b]);
    assert_eq!(stage.roots().len(), 3, "membership is not duplicated");
}

#[test]
fn bring_to_back_beats_z_index_until_the_next_add() {
    // R-7, the sharp edge, pinned deliberately: `bring_to_back` demotes
    // regardless of z_index (re-sorting there would make the call a no-op),
    // and the next `add` renormalizes.
    let mut stage = Stage::new();
    let low = dot(&mut stage);
    let high = dot(&mut stage);
    stage.set_z_index(high, 100, true);
    for m in [low, high] {
        stage.add_to_scene(m).expect("root");
    }
    assert_eq!(stage.draw_plan().sequence(), [low, high]);

    stage.bring_to_back(high).expect("demote");
    assert_eq!(
        stage.draw_plan().sequence(),
        [high, low],
        "the demotion holds even though z_index says otherwise"
    );

    let newcomer = dot(&mut stage);
    stage.add_to_scene(newcomer).expect("root");
    assert_eq!(
        stage.draw_plan().sequence(),
        [low, newcomer, high],
        "the next add renormalizes by z_index"
    );
}

#[test]
fn removing_a_family_member_ungroups_its_ancestor_in_place() {
    // R-8 (scene.py:371 over family_ops.py:23): the group's other children
    // take its place, spliced, in order — and nothing is deleted.
    let mut stage = Stage::new();
    let first = dot(&mut stage);
    let x = dot(&mut stage);
    let y = dot(&mut stage);
    let z = dot(&mut stage);
    let last = dot(&mut stage);
    let trio = group(&mut stage, &[x, y, z]);
    for m in [first, trio, last] {
        stage.add_to_scene(m).expect("root");
    }
    assert_eq!(stage.draw_plan().sequence(), [first, x, y, z, last]);

    stage.remove_from_scene(y);
    assert_eq!(
        stage.roots(),
        [first, x, z, last],
        "the survivors are spliced into the group's position, ungrouped"
    );
    assert_eq!(stage.draw_plan().sequence(), [first, x, z, last]);
    assert!(
        stage.contains(y),
        "removal is a draw-list edit, not a delete"
    );
    assert!(stage.contains(trio));
}

#[test]
fn removing_a_nested_member_ungroups_only_the_branch_it_is_in() {
    let mut stage = Stage::new();
    let a = dot(&mut stage);
    let b = dot(&mut stage);
    let c = dot(&mut stage);
    let inner = group(&mut stage, &[a, b]);
    let outer = group(&mut stage, &[inner, c]);
    stage.add_to_scene(outer).expect("root");
    assert_eq!(stage.draw_plan().sequence(), [a, b, c]);

    stage.remove_from_scene(b);
    // outer is replaced by its children with the inner branch ungrouped in
    // place: [inner's survivors..., c].
    assert_eq!(stage.roots(), [a, c]);
    assert_eq!(stage.draw_plan().sequence(), [a, c]);
}

#[test]
fn replace_splices_in_place_and_only_for_members() {
    // R-7 / scene.py:360.
    let mut stage = Stage::new();
    let a = dot(&mut stage);
    let b = dot(&mut stage);
    let c = dot(&mut stage);
    let x = dot(&mut stage);
    let y = dot(&mut stage);
    for m in [a, b, c] {
        stage.add_to_scene(m).expect("root");
    }
    stage.replace_in_scene(b, &[x, y]).expect("replace");
    assert_eq!(stage.draw_plan().sequence(), [a, x, y, c]);

    // A member that is not on stage is left alone.
    let stranger = dot(&mut stage);
    stage.replace_in_scene(stranger, &[a]).expect("no-op");
    assert_eq!(stage.draw_plan().sequence(), [a, x, y, c]);
}

// ----------------------------------------------------------- the families

#[test]
fn families_draw_depth_first_with_pointless_members_skipped() {
    // mobject.py:2056 over family_members_with_points (mobject.py:435).
    let mut stage = Stage::new();
    let leaf_a = dot(&mut stage);
    let leaf_b = dot(&mut stage);
    let inner = group(&mut stage, &[leaf_a, leaf_b]); // pointless container
    let leaf_c = dot(&mut stage);
    let outer = group(&mut stage, &[inner, leaf_c]);
    stage.add_to_scene(outer).expect("root");

    let plan = stage.draw_plan();
    assert_eq!(
        plan.sequence(),
        [leaf_a, leaf_b, leaf_c],
        "containers carry no records and never draw"
    );
    assert!(
        !plan.sequence().contains(&outer) && !plan.sequence().contains(&inner),
        "a pointless member is skipped, not drawn empty"
    );
}

#[test]
fn a_bare_mobjects_points_never_draw_and_end_the_batch() {
    // R-13: the Reference's base Mobject has an empty shader_folder
    // (mobject.py:69), so its wrapper compiles no program
    // (shader_wrapper.py:83-88) but still carries its own shader id.
    let mut stage = Stage::new();
    let before = dot(&mut stage);
    let bare = stage.add(Mobject::from_points(&[[0.0; 3], [1.0, 0.0, 0.0]]));
    let after = dot(&mut stage);
    let root = group(&mut stage, &[before, bare, after]);
    stage.add_to_scene(root).expect("root");
    let plan = stage.draw_plan();
    assert_eq!(plan.sequence(), [before, after], "bare points are data");
    assert_eq!(
        plan.batch_trace(),
        [0, 1],
        "the bare member splits the batch"
    );

    // Control: the same record as a vector mobject draws, in one batch.
    let mut stage = Stage::new();
    let before = dot(&mut stage);
    let vector = stage.add(Mobject::vector_from_points(&[[0.0; 3], [1.0, 0.0, 0.0]]));
    let after = dot(&mut stage);
    let root = group(&mut stage, &[before, vector, after]);
    stage.add_to_scene(root).expect("root");
    let plan = stage.draw_plan();
    assert_eq!(plan.sequence(), [before, vector, after]);
    assert_eq!(plan.batch_trace(), [0, 0, 0]);
}

#[test]
fn a_diamond_child_draws_once_inside_one_family() {
    // R-11, the deliberate divergence: our family dedups, the Reference's
    // get_family concatenates and would draw the shared child twice.
    let mut stage = Stage::new();
    let shared = dot(&mut stage);
    let left = group(&mut stage, &[shared]);
    let right = group(&mut stage, &[shared]);
    let root = group(&mut stage, &[left, right]);
    stage.add_to_scene(root).expect("root");
    assert_eq!(stage.draw_plan().sequence(), [shared]);
}

#[test]
fn a_child_of_two_roots_draws_once_per_root() {
    // R-12: roots added in the same Reference Scene.add call are two
    // placements, each reported with its own root.
    let mut stage = Stage::new();
    let shared = dot(&mut stage);
    let left = group(&mut stage, &[shared]);
    let right = group(&mut stage, &[shared]);
    stage
        .add_many_to_scene(&[left, right])
        .expect("batch roots");

    let plan = stage.draw_plan();
    assert_eq!(plan.sequence(), [shared, shared]);
    let roots: Vec<Mob> = plan.items().iter().map(|item| item.root).collect();
    assert_eq!(roots, [left, right]);
}

#[test]
fn a_later_parent_add_removes_its_already_rooted_child() {
    // scene.py:333 expands the newly added mobject's complete family before
    // removing it from the existing scene list.
    let mut stage = Stage::new();
    let child = dot(&mut stage);
    stage.add_to_scene(child).expect("child root");
    let parent = group(&mut stage, &[child]);
    stage.add_to_scene(parent).expect("parent root");

    assert_eq!(stage.roots(), [parent]);
    assert_eq!(
        stage.draw_plan().sequence(),
        [child],
        "the child has one placement through its new parent"
    );
}

#[test]
fn a_childs_z_index_orders_nothing() {
    // R-9: the scene sort reads the draw list, and a child is not in it.
    let mut stage = Stage::new();
    let first = dot(&mut stage);
    let second = dot(&mut stage);
    let parent = group(&mut stage, &[first, second]);
    stage.set_z_index(second, -100, false);
    stage.add_to_scene(parent).expect("root");
    assert_eq!(
        stage.draw_plan().sequence(),
        [first, second],
        "a child's z_index cannot reorder its siblings"
    );
}

#[test]
fn set_z_index_recurses_by_default_but_does_not_resort() {
    // R-9 / R-10 (mobject.py:1238).
    let mut stage = Stage::new();
    let child = dot(&mut stage);
    let parent = group(&mut stage, &[child]);
    let other = dot(&mut stage);
    for m in [parent, other] {
        stage.add_to_scene(m).expect("root");
    }
    stage.set_z_index(parent, 7, true);
    assert_eq!(stage.z_index(parent), 7);
    assert_eq!(
        stage.z_index(child),
        7,
        "the write recurses over the family"
    );
    assert_eq!(
        stage.draw_plan().sequence(),
        [child, other],
        "the setter does not renormalize the scene"
    );
    stage.bring_to_front(parent).expect("renormalize");
    assert_eq!(stage.draw_plan().sequence(), [other, child]);

    stage.set_z_index(parent, 0, false);
    assert_eq!(stage.z_index(child), 7, "recurse=false writes one entry");
}

// -------------------------------------------------------------- batching

#[test]
fn adjacent_compatible_items_share_one_batch() {
    let mut stage = Stage::new();
    let mobs: Vec<Mob> = (0..4).map(|_| dot(&mut stage)).collect();
    let parent = group(&mut stage, &mobs);
    stage.add_to_scene(parent).expect("root");

    let plan = stage.draw_plan();
    assert_eq!(plan.batch_trace(), [0, 0, 0, 0]);
    assert_eq!(plan.batch_count(), 1);
}

#[test]
fn an_incompatible_uniform_splits_the_batch_and_rejoins_it() {
    let mut stage = Stage::new();
    let plain_a = dot(&mut stage);
    let depth = styled(
        &mut stage,
        Uniforms {
            depth_test: true,
            ..Uniforms::default()
        },
    );
    let plain_b = dot(&mut stage);
    let plain_c = dot(&mut stage);
    let parent = group(&mut stage, &[plain_a, depth, plain_b, plain_c]);
    stage.add_to_scene(parent).expect("root");

    let plan = stage.draw_plan();
    assert_eq!(plan.sequence(), [plain_a, depth, plain_b, plain_c]);
    assert_eq!(
        plan.batch_trace(),
        [0, 1, 2, 2],
        "R-4: depth_test partitions the call, it does not reorder"
    );
    assert_eq!(plan.batch_count(), 3);
}

#[test]
fn every_uniform_that_the_reference_hashes_splits_a_batch() {
    // R-3: the key is exactly the shader-id material. One test per lever, so
    // a future uniform that quietly stops splitting is caught here.
    let levers: Vec<(&str, Uniforms)> = vec![
        (
            "is_fixed_in_frame",
            Uniforms {
                is_fixed_in_frame: 1.0,
                ..Uniforms::default()
            },
        ),
        (
            "shading",
            Uniforms {
                shading: [0.3, 0.2, 0.4],
                ..Uniforms::default()
            },
        ),
        (
            "anti_alias_width",
            Uniforms {
                anti_alias_width: 1.5 + f64::EPSILON,
                ..Uniforms::default()
            },
        ),
        (
            "joint_type",
            Uniforms {
                joint_type: JointType::Bevel,
                ..Uniforms::default()
            },
        ),
        (
            "flat_stroke",
            Uniforms {
                flat_stroke: true,
                ..Uniforms::default()
            },
        ),
        (
            "scale_stroke_with_zoom",
            Uniforms {
                scale_stroke_with_zoom: true,
                ..Uniforms::default()
            },
        ),
        (
            "stroke_behind",
            Uniforms {
                stroke_behind: true,
                ..Uniforms::default()
            },
        ),
        (
            "depth_test",
            Uniforms {
                depth_test: true,
                ..Uniforms::default()
            },
        ),
        ("clip_planes", {
            let mut u = Uniforms::default();
            u.clip_planes[0] = [0.0, 1.0, 0.0, 0.0];
            u
        }),
    ];
    for (name, uniforms) in levers {
        let mut stage = Stage::new();
        let plain = dot(&mut stage);
        let different = styled(&mut stage, uniforms);
        let parent = group(&mut stage, &[plain, different]);
        stage.add_to_scene(parent).expect("root");
        assert_eq!(
            stage.draw_plan().batch_trace(),
            [0, 1],
            "{name} must split the batch"
        );
    }
}

#[test]
fn a_batch_never_crosses_a_group_boundary() {
    // R-1: two adjacent top-level members whose keys agree but whose
    // z_index differs are two render groups, so their identical-key family
    // members cannot merge into one call.
    let mut stage = Stage::new();
    let back = dot(&mut stage);
    let front = dot(&mut stage);
    stage.set_z_index(front, 1, true);
    for m in [back, front] {
        stage.add_to_scene(m).expect("root");
    }

    let plan = stage.draw_plan();
    assert_eq!(plan.sequence(), [back, front]);
    assert_eq!(plan.group_count(), 2, "z_index splits the render groups");
    assert_eq!(
        plan.batch_trace(),
        [0, 1],
        "identical keys still cannot share a call across groups"
    );

    // The same two objects at one z_index are one group and one call.
    let mut stage = Stage::new();
    let a = dot(&mut stage);
    let b = dot(&mut stage);
    for m in [a, b] {
        stage.add_to_scene(m).expect("root");
    }
    let plan = stage.draw_plan();
    assert_eq!(plan.group_count(), 1);
    assert_eq!(plan.batch_trace(), [0, 0]);
}

#[test]
fn stroke_behind_is_reported_per_item_as_the_pass_order() {
    // R-5 (shader_wrapper.py:277).
    let mut stage = Stage::new();
    let normal = dot(&mut stage);
    let behind = styled(
        &mut stage,
        Uniforms {
            stroke_behind: true,
            ..Uniforms::default()
        },
    );
    let parent = group(&mut stage, &[normal, behind]);
    stage.add_to_scene(parent).expect("root");

    let plan = stage.draw_plan();
    let passes: Vec<PassOrder> = plan.items().iter().map(|item| item.passes).collect();
    assert_eq!(
        passes,
        [PassOrder::FillThenStroke, PassOrder::StrokeThenFill]
    );
    assert_eq!(plan.batch_trace(), [0, 1], "and it splits the call");
}

#[test]
fn the_program_kind_is_part_of_the_key() {
    let mut stage = Stage::new();
    let vector = dot(&mut stage);
    let surface = stage.add(
        Mobject::from_points(&[[0.0, 0.0, 0.0]])
            .with_render_primitive(RenderPrimitive::SurfaceGrid { resolution: (1, 1) }),
    );
    let dots = stage.add(
        Mobject::from_points(&[[0.0, 0.0, 0.0]]).with_render_primitive(RenderPrimitive::DotCloud),
    );
    for mob in [vector, surface, dots] {
        stage.add_to_scene(mob).expect("root");
    }
    let plan = stage.draw_plan();
    assert_eq!(
        plan.items()
            .iter()
            .map(|item| item.key.program)
            .collect::<Vec<_>>(),
        [
            ProgramKind::Vector,
            ProgramKind::Surface,
            ProgramKind::DotCloud
        ]
    );
    assert_eq!(plan.batch_trace(), [0, 1, 2]);
}

// ------------------------------------------------------------- properties

#[test]
fn the_plan_is_stable_under_no_op_mutations() {
    // R-2: a pure function of scene state. Re-plan after operations that
    // change nothing about order, and the trace must be identical.
    let mut stage = Stage::new();
    let a = dot(&mut stage);
    let b = dot(&mut stage);
    let c = dot(&mut stage);
    let parent = group(&mut stage, &[b, c]);
    for m in [a, parent] {
        stage.add_to_scene(m).expect("root");
    }
    let before = stage.draw_plan();

    // Idempotent re-plan.
    assert_eq!(stage.draw_plan().sequence(), before.sequence());
    assert_eq!(stage.draw_plan().batch_trace(), before.batch_trace());

    // Point writes move geometry, never order.
    if let Some(entry) = stage.get_mut(b) {
        entry.buffer.write(0, "point", &[3.0, 4.0, 0.0]);
    }
    assert_eq!(stage.draw_plan().sequence(), before.sequence());

    // Removing something that was never on stage.
    let stranger = dot(&mut stage);
    stage.remove_from_scene(stranger);
    assert_eq!(stage.draw_plan().sequence(), before.sequence());

    // Setting a z_index to the value it already has.
    stage.set_z_index(a, 0, true);
    assert_eq!(stage.draw_plan().sequence(), before.sequence());
}

#[test]
fn a_snapshot_round_trip_preserves_the_order() {
    // Order is scene state, so it travels with a snapshot (§8.1) — and the
    // §6.7 minor bump that added z_index to the durable form is what makes
    // that true across a save.
    let mut stage = Stage::new();
    let low = dot(&mut stage);
    let high = dot(&mut stage);
    stage.set_z_index(high, 3, true);
    stage.set_z_index(low, -3, true);
    for m in [high, low] {
        stage.add_to_scene(m).expect("root");
    }
    let expected = stage.draw_plan().sequence();
    assert_eq!(expected, [low, high]);

    let snapshot = stage.snapshot();
    stage.set_z_index(high, 0, true);
    assert_eq!(stage.z_index(high), 0);
    stage.restore(&snapshot);
    assert_eq!(stage.z_index(high), 3);
    assert_eq!(stage.draw_plan().sequence(), expected);
}

#[test]
fn an_empty_scene_plans_nothing() {
    let stage = Stage::new();
    let plan = stage.draw_plan();
    assert!(plan.items().is_empty());
    assert_eq!(plan.batch_count(), 0);
    assert_eq!(plan.group_count(), 0);
}

#[test]
fn a_stale_handle_is_refused_rather_than_ordered() {
    let mut stage = Stage::new();
    let mob = dot(&mut stage);
    stage.add_to_scene(mob).expect("root");
    stage.delete(mob).expect("delete");
    assert!(stage.add_to_scene(mob).is_err());
    assert!(stage.bring_to_back(mob).is_err());
    assert!(stage.draw_plan().items().is_empty());
}
