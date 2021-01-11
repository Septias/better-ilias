<template>
  <span
    v-if="node.visible || edit_visibility"
    class="p-1 rounded-sm hover:bg-accent text-white select-none"
    :class="{ 'text-opacity-25': !node.visible && edit_visibility }"
  >
    <svg
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
  emits: ["set_invisible", "set_visible"],
  props: {
    node: {
      type: Object,
      required: true,
    },
    index: {
      type: Number,
      required: true,
    },
    color: {
      type: String,
      required: true,
    },
  },
  setup(props, context) {
    let { edit_visibility } = useVisibility();
    function handle_click() {
      if (edit_visibility.value) {
        if (props.node.visible) {
          context.emit("set_invisible", [props.index]);
        } else {
          context.emit("set_visible", [props.index]);
        }
      }
    }

    return { edit_visibility, handle_click };
  },
};
</script>