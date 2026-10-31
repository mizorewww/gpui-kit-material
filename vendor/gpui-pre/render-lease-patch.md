# Entity lease for rendering

Source: the published Apache-2.0 `gpui-pre` 0.3.10 package (`SOURCE.json`) and
`gpui-fast` 0.1.12 (`MATERIAL-MIGRATION.json`). Files: `src/app/entity_map.rs` here,
`src/fast/dependencies.rs` in `gpui-fast`.

`Entity::update_for_render` leases an entity to build elements from it. It exposes
what `gpui-fast` already does when the platform queries a text input
(`dependencies::query`): the update is not stamped as a write, so a retained subtree
that read the entity stays valid. A notify during the lease still stamps it. Here, with
no retained subtrees, it is `update`.

Rows that need an entity's `Context` for their listeners had only `update`, and each
row built that way invalidated every retained row that read the entity. Depends on no
other local patch.
