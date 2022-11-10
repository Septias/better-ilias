import { invoke } from '@tauri-apps/api'

export const get_breed = function (breed: any) {
  if (typeof breed == 'object') {
    return Object.keys(breed)[0]
  }
  else {
    return breed
  }
}

export async function invoke_log(command: string, args: any = undefined): Promise<any> {
  console.log('invoke', command, args)
  try { return await invoke(command, args) }
  catch (e) { console.warn(e) }
}
