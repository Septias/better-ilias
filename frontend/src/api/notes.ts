import axios from 'axios'
import { Note } from '../components/compositions';

export function create_note(note: Note) {
  return axios.post('/api/notes/create', note)
}

export async function fetch_notes(): Promise<Note[]> {
  return await (await axios.get('api/notes/list')).data as Note[]
}

export function update_note(note: Note) {
  return axios.post('/api/notes/update', note).then((e) => console.debug(e.data))
}