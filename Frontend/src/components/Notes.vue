<template>
  <div class="h-full flex flex-col p-4 pt-2">
    <div>
      <button
        v-for="note in notes"
        :key="note.course"
        class="p-1 mr-2 border-accent border-b rounded focus:outline-none select-none cursor-pointer"
        :class="{ 'bg-light-main': active.uri == note.uri }"
      >
        <span @click="active = note"> {{ note.course }} </span>

        <mdi-close
          class="inline-block hover:text-accent"
          @click="
            () => {
              hide_note(note.uri);
            }
          "
        />
      </button>
      <codicon-chrome-minimize
        class="inline-block float-right mt-2 text-accent hover:bg-light-main cursor-pointer"
        @click="reset_note"
      />
    </div>
    <textarea
      class="bg-light-main mt-2 flex-grow rounded p-2 overflow-y-auto"
      v-model="active.body"
      placeholder="Notizen"
    ></textarea>
  </div>
</template>

<script>
import { useNotes } from "./compositions";

export default {
  name: "Notes",

  async setup(props) {
    const { reset_note, active, get_notes, hide_note } = useNotes();

    let notes = await get_notes();

    return {
      reset_note,
      active,
      notes,
      hide_note,
      reset_note,
    };
  },
};
</script>