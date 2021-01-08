

enum IlNodeType {
    Forum,
    Folder,
    DirectLink,
    File,
}

interface IlNode {
    title: String,
    id: number,
    uri: String,
    breed: IlNodeType,
    children?: [IlNode],
    parent: number
}
