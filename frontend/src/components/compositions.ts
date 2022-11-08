import { computed, ref } from "vue"
import { create_note, fetch_notes, update_note } from "../api/notes"
import { IlNode } from "./IlTypes"





let all_notes = ref([] as Note[])
let active = ref(undefined as Note | undefined)
let visible_notes = ref([] as String[]); // TODO: Make this a set

export interface Note {
  body: string,
  uri: string,
  course: string,
}

let notes = computed(() => {
  return all_notes.value.filter((a) => visible_notes.value.find((uri) => uri == a.uri))
})

let loaded = false
async function get_notes() {
  if (!loaded) {
    all_notes.value.push(...(await fetch_notes()).map((note: Note) => new Proxy(note, handler)));
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
    timeout = setTimeout(() => {
      update_note(obj)
    }, 750)

    //obj[prop as keyof typeof Note] = val
    return Reflect.set(obj, prop, val)
  }
} as ProxyHandler<Note>


async function activate_note(node: IlNode) {
  await get_notes() // make sure notes from server are loaded
  const note = all_notes.value.find((note) => note.uri == node.uri);
  if (!note) {
    let new_note = new Proxy({
      uri: node.uri,
      course: node.title.slice(0, 14) + '...',
      body: ""
    }, handler) as Note

    let resp = await create_note(new_note)
    if (resp.status == 201) {
      all_notes.value.push(new_note)
      visible_notes.value.push(new_note.uri)
      active.value = new_note
    }
  } else {
    active.value = note
    visible_notes.value.push(node.uri)
  }
}
const reset_note = () => {
  active.value = undefined
}

const hide_note = (note_uri: string) => {
  const index = visible_notes.value.indexOf(note_uri);
  visible_notes.value.splice(index, 1)

  if (note_uri == active.value?.uri) {
    if (notes.value.length > 0) {
      active.value = notes.value[0]
    } else {
      active.value = undefined
    }
  }
}


export function useNotes() {
  return {
    activate_note,
    reset_note,
    active,
    get_notes,
    hide_note
  }
}

