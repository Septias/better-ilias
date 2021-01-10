<template>
  <span
    class="p-1 rounded-sm hover:bg-accent bg-opacity-25 text-white"
    :class="{ 'text-opacity-25': !node.visible && edit_visibility }"
    @click="handle_click"
  >
    <SelectBox v-if="edit_visibility" :ckecked="node.visibile"></SelectBox>
    <svg
      v-else
      :class="color"
      class="hover:text-white fill-current inline"
      focusable="false"
      width="1em"
      height="1em"
      viewBox="0 0 24 24"
    >
      <slot></slot>
    </svg>
    {{ node.title }}
  </span>
</template>

<script>
import { useVisibility } from "./compositions";

export default {
  name: "File",
  emits: ["set_invisible"],
  props: {
    node: Object,
    index: Number,
    color: String,
  },
  setup(props, context) {
    let { edit_visibility } = useVisibility();
    function handle_click() {
      if (edit_visibility.value) {
        context.emit("set_invisible", [props.index]);
      }
    }
    return { edit_visibility, handle_click };
  },
};
</script>