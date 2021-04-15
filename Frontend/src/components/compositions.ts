import { computed, ref } from "vue"
import { create_note, fetch_notes, update_note } from "../api/notes"
import { IlNode } from "./IlTypes"


let edit_visibility = ref(false)

export function useVisibility() {
  return { edit_visibility }
}


let notes = ref([] as Note[])
let active = ref(undefined as Note | undefined)

export interface Note {
  body: string,
  uri: string,
  course: string,
}

let loaded = false
async function get_notes() {
  if (!loaded) {
    notes.value.push(...(await fetch_notes()).map((note: Note) => new Proxy(note, handler)));
    loaded = true
    return notes
  }
  return notes
}

// this is a kinda cheeky implementation since all Note-objects share the same update-timeout
let timeout: undefined | number = undefined

let handler = {
  set: function (obj: Note, prop: string | symbol, val: any, receiver: any) {

    if (timeout) {
      clearTimeout(timeout)
    }
    console.log("update for", obj.course)
    timeout = setTimeout(() => {
      update_note(obj)
    }, 750)

    //obj[prop as keyof typeof Note] = val
    return Reflect.set(obj, prop, val)
  }
} as ProxyHandler<Note>


async function activate_note(node: IlNode) {
  await get_notes() // make sure notes from server are loaded
  const note = notes.value.find((note) => note.uri == node.uri);
  if (!note) {
    let new_note = new Proxy({
      uri: node.uri,
      course: node.title.slice(0, 14) + '...',
      body: ""
    }, handler) as Note

    let resp = await create_note(new_note)
    if (resp.status == 201) {
      notes.value.push(new_note)
      active.value = new_note
    }
  } else {
    active.value = note
  }
}
const reset_note = () => {
  active.value = undefined
}

export function useNotes() {
  return {
    activate_note,
    reset_note,
    active,
    get_notes
  }
}

