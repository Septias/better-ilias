<script setup>
const props = defineProps({
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
  isLocal: {
    type: Boolean,
    default: false,
  },
})

const emit = defineEmits(['set_invisible', 'set_visible'])
function handle_click() {
  if (edit_visibility.value) {
    if (props.node.visible) {
      context.emit('set_invisible', [props.index])
    }
    else {
      context.emit('set_visible', [props.index])
    }
  }
}
</script>

<template>
  <span v-if="node.visible || edit_visibility" class="p-1 rounded-sm hover:bg-accent text-white select-none"
    :class="{ 'text-opacity-25': !node.visible && edit_visibility }" @click="handle_click">
    <svg :class="color" class="hover:text-white fill-current inline" focusable="false" width="1em" height="1em"
      viewBox="0 0 24 24">
      <slot name="default" />
    </svg>
    <slot name="body">
      {{ node.title }}
    </slot>
  </span>
</template>
