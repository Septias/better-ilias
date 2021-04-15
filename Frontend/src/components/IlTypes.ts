

export enum IlNodeType {
  Forum,
  Folder,
  DirectLink,
  File,
}

export interface IlNode {
  title: String,
  id: number,
  uri: String,
  breed: IlNodeType,
  children?: [IlNode],
  parent: number,
  visible: boolean
}
