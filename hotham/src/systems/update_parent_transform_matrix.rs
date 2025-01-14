use std::collections::HashMap;

use crate::components::{Parent, TransformMatrix};
use hecs::{Entity, PreparedQuery, Without, World};

use nalgebra::Matrix4;

/// Update parent transform matrix system
/// Walks through each entity that has a Parent and builds a hierarchy
/// Then transforms each entity based on the hierarchy
pub fn update_parent_transform_matrix_system(
    parent_query: &mut PreparedQuery<&Parent>,
    roots_query: &mut PreparedQuery<Without<Parent, &TransformMatrix>>,
    world: &mut World,
) {
    // Build hierarchy
    let mut hierarchy: HashMap<Entity, Vec<Entity>> = HashMap::new();
    for (entity, parent) in parent_query.query_mut(world) {
        let children = hierarchy.entry(parent.0).or_default();
        children.push(entity);
    }

    let mut roots = roots_query.query(world);
    for (root, root_matrix) in roots.iter() {
        update_transform_matrix(&root_matrix.0, root, &hierarchy, world);
    }
}

fn update_transform_matrix(
    parent_matrix: &Matrix4<f32>,
    entity: Entity,
    hierarchy: &HashMap<Entity, Vec<Entity>>,
    world: &World,
) {
    if let Some(children) = hierarchy.get(&entity) {
        for child in children {
            {
                let child_matrix = &mut world.get_mut::<TransformMatrix>(*child).unwrap().0;
                *child_matrix = parent_matrix * *child_matrix;
            }
            let child_matrix = world.get::<TransformMatrix>(*child).unwrap().0;
            update_transform_matrix(&child_matrix, *child, hierarchy, world);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use approx::{assert_relative_eq, relative_eq};
    use nalgebra::vector;

    use crate::{
        components::{Info, Transform},
        systems::update_transform_matrix_system,
    };

    use super::*;
    #[test]
    pub fn test_transform_system() {
        let mut world = World::new();
        let parent_transform_matrix =
            TransformMatrix(Matrix4::new_translation(&vector![1.0, 1.0, 100.0]));

        let parent = world.spawn((parent_transform_matrix,));
        let child = world.spawn((parent_transform_matrix, Parent(parent)));
        let grandchild = world.spawn((parent_transform_matrix, Parent(child)));

        schedule(&mut world);

        {
            let transform_matrix = world.get_mut::<TransformMatrix>(grandchild).unwrap();
            let expected_matrix = Matrix4::new_translation(&vector![3.0, 3.0, 300.0]);
            assert_relative_eq!(transform_matrix.0, expected_matrix);
        }

        {
            let transform_matrix = world.get_mut::<TransformMatrix>(child).unwrap();
            let expected_matrix = Matrix4::new_translation(&vector![2.0, 2.0, 200.0]);
            assert_relative_eq!(transform_matrix.0, expected_matrix);
        }
    }

    #[test]
    pub fn test_transform_system_extensive() {
        let mut world = World::new();
        let mut hierarchy: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut node_entity: HashMap<usize, Entity> = HashMap::new();
        let mut entity_node: HashMap<Entity, usize> = HashMap::new();
        hierarchy.insert(0, vec![1, 2, 3, 4]);
        hierarchy.insert(1, vec![5, 6, 7, 8]);
        hierarchy.insert(2, vec![9, 10, 11, 12]);
        hierarchy.insert(3, vec![13, 14, 15, 16]);
        hierarchy.insert(5, vec![17, 18, 19, 20]);
        hierarchy.insert(14, vec![21, 22, 23, 24]);
        hierarchy.insert(22, vec![25, 26, 27, 28]);
        hierarchy.insert(17, vec![29, 30, 31, 32]);

        for n in 0..=32 {
            let info = Info {
                name: format!("Node {}", n),
                node_id: n,
            };
            let transform = Transform {
                translation: vector![1.0, 1.0, 1.0],
                ..Default::default()
            };
            let matrix = TransformMatrix::default();
            let entity = world.spawn((info, transform, matrix));
            node_entity.insert(n, entity);
            entity_node.insert(entity, n);
        }

        for (parent, children) in hierarchy.iter() {
            let parent_entity = node_entity.get(parent).unwrap();
            let parent = Parent(*parent_entity);
            for node_id in children {
                let entity = node_entity.get(node_id).unwrap();
                world.insert_one(*entity, parent).unwrap();
            }
        }

        let root_entity = node_entity.get(&0).unwrap();
        {
            let mut transform = world.get_mut::<Transform>(*root_entity).unwrap();
            transform.translation = vector![100.0, 100.0, 100.0];
        }
        schedule(&mut world);

        for (_, (transform_matrix, parent, info)) in
            world.query::<(&TransformMatrix, &Parent, &Info)>().iter()
        {
            let mut depth = 1;

            let mut parent_entity = parent.0;
            let mut parent_matrices = vec![];
            loop {
                let parent_transform_matrix = world.get::<TransformMatrix>(parent_entity).unwrap();
                parent_matrices.push(parent_transform_matrix.0);

                // Walk up the tree until we find the root.
                if let Ok(grand_parent) = world.get::<Parent>(parent_entity) {
                    depth += 1;
                    parent_entity = grand_parent.0;
                } else {
                    let expected_matrix = get_expected_matrix(depth);
                    if !relative_eq!(expected_matrix, transform_matrix.0) {
                        panic!(
                            "[Node {}] - {:?} did not equal {:?} at depth {}",
                            info.node_id, transform_matrix.0, expected_matrix, depth
                        );
                    }
                    break;
                }
            }
        }
    }

    fn get_expected_matrix(depth: usize) -> Matrix4<f32> {
        let mut transform = Matrix4::new_translation(&vector![100.0, 100.0, 100.0]);
        for _ in 0..depth {
            transform *= Matrix4::new_translation(&vector![1.0, 1.0, 1.0]);
        }
        transform
    }

    #[test]
    pub fn test_entities_without_transforms() {
        let mut world = World::new();
        world.spawn((0,));
        schedule(&mut world);
    }

    fn schedule(world: &mut World) {
        update_transform_matrix_system(&mut Default::default(), world);
        update_parent_transform_matrix_system(
            &mut Default::default(),
            &mut Default::default(),
            world,
        );
    }
}
