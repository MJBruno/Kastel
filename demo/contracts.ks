export interface Base {}
export interface Marker: Base {}

export class Good: Marker {}
export class Bad {}

export func identity<T: Base>(value: T) -> T {
    return value;
}