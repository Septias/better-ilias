<template>
  <div class="h-full flex flex-col p-3">
    <div>
      <button
        v-for="note in notes"
        :key="note.course"
        class="p-1 mr-2 border-accent border-b rounded focus:outline-none select-none cursor-pointer"
        :class="{ 'bg-light-main': active.uri == note.uri }"
        @click="active = note"
      >
        {{ note.course }}
      </button>
      <codicon-chrome-minimize
        class="inline-block float-right mt-2 text-accent hover:bg-light-main cursor-pointer"
        @click="reset_note"
      />
    </div>
    <textarea
      class="bg-light-main mt-3 flex-grow rounded p-2 overflow-y-auto"
      v-model="active.body"
      placeholder="Notizen"
    ></textarea>
  </div>
</template>

<script>
import axios from "axios";
import { ref } from "vue";
import { useNotes } from "./compositions";

const mock_nodes = [
  { course: "Rechnernetze", uri: "htt", body: "a" },
  { course: "Programmieren", uri: "ppt", body: "b" },
];

export default {
  name: "Notes",

  setup(props) {
    const { activate_note, reset_note, active, notes } = useNotes();

    return {
      reset_note,
      active,
      notes,
    };
  },
};
</script>