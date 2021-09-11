pub trait Lattice {
    type Array;
    type Value;

    const SIZE: usize;
    const ELEMENTS: usize;

    fn array(&self) -> &Self::Array;
    fn array_(&mut self) -> &mut Self::Array;
    fn set(&mut self, x: usize, y: usize, z: usize, value: Self::Value);
    fn get(&self, x: usize, y: usize, z: usize) -> Self::Value;
}
