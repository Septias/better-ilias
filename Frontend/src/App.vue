<template>
  <div class="flex flex-col h-screen">
    <div
      class="border-accent"
      :class="{ 'flex-grow': !note_panel, test: note_panel }"
    >
      <Suspense>
        <Ilias />
      </Suspense>
    </div>
    <div v-if="note_panel" class="flex-grow flex-shrink overflow-y-auto">
      <Notes></Notes>
    </div>
  </div>
</template>

<style lang="sass">
.test
  border-bottom: 2px solid
  resize: vertical
  overflow: auto
</style>

<script lang="ts">
import { computed, defineComponent, ref } from "vue";
import { useNotes } from "./components/compositions";

export default defineComponent({
  name: "App",
  setup() {
    const { activate_note, reset_note, active, notes } = useNotes();

    // prob useless
    const note_panel = computed(() => {
      return active.value != undefined;
    });

    return {
      note_panel,
    };
  },
});
</script>

<style>
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  color: #fcfcfc;
}
:root {
  background: #15152b;
}

*:focus {
  @apply outline-accent;
}
</style>