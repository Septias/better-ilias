import { type ViteSSGContext } from 'vite-ssg'

export type UserModule = (ctx: ViteSSGContext) => void

export enum IlNodeType {
  Forum = 'Forum',
  Folder = 'Folder',
  DirectLink = 'DirectLink',
  File = 'File',
}

export interface IlNode {
  title: String
  id: number
  uri: String
  breed: IlNodeType
  children?: [IlNode]
  parent: number
  visible: boolean
}
