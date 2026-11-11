pub trait Visitor<T> {
    fn visit(&mut self, value: &mut T);
}

/// Get the last ident of a use path (not the `as` rename ident!)
pub fn use_path_last_ident(path: &mut syn::UseTree) -> Option<&mut syn::Ident> {
    match path {
        syn::UseTree::Path(syn::UsePath { tree, .. }) => use_path_last_ident(tree),
        syn::UseTree::Name(syn::UseName { ident }) => Some(ident),
        syn::UseTree::Rename(syn::UseRename { ident, .. }) => Some(ident),
        syn::UseTree::Glob(_) => None,
        syn::UseTree::Group(_) => None,
    }
}

pub fn path_last_ident(path: &mut syn::Path) -> Option<&mut syn::Ident> {
    path.segments.last_mut().map(|seg| &mut seg.ident)
}

#[derive(Debug)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

pub trait Partition: Iterator {
    fn partition_with<F, L, R, Left, Right>(self, f: F) -> (Left, Right)
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Either<L, R>,
        Left: Default + Extend<L>,
        Right: Default + Extend<R>,
    {
        let (mut left, mut right) = (Left::default(), Right::default());
        for item in self.map(f) {
            match item {
                Either::Left(l) => left.extend(std::iter::once(l)),
                Either::Right(r) => right.extend(std::iter::once(r)),
            }
        }

        (left, right)
    }
}

impl<T, Iter> Partition for Iter where Iter: Iterator<Item = T> {}
