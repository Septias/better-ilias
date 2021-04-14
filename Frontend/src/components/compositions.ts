import { computed, ref } from "vue"


let edit_visibility = ref(false)

export function useVisibility() {

  return { edit_visibility }
}


let notes = ref([] as Note[])
let active = ref(undefined as Note | undefined)

interface Note {
  body: string,
  uri: string,
  course: string,
}

interface Node {
  uri: String,
  title: String
}

const activate_note = (node: Node) => {
  const note = notes.value.find((note) => note.uri == node.uri);
  if (!note) {
    let new_note = {
      uri: node.uri,
      course: node.title.slice(0, 14) + '...',
      body: ""
    } as Note
    notes.value.push(new_note)
    active.value = new_note
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
    notes
  }
}

