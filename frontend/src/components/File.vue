<script setup lang="ts">
import { invoke_log } from '~/utils'

const props = defineProps({
  node: {
    type: Object,
    required: true,
  },
})

async function open_file() {
  await invoke_log('open', { path: props.node.breed.File.path })
}
</script>

<template>
  <Leaf :node="node" color="text-blue-500" :is-local="node.breed.File.local" @dblclick="open_file">
    <path
      d="M14 11a3 3 0 0 1-3-3V4H7a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2h9a2 2 0 0 0 2-2v-8h-4zm-2-3a2 2 0 0 0 2 2h3.586L12 4.414V8zM7 3h5l7 7v9a3 3 0 0 1-3 3H7a3 3 0 0 1-3-3V6a3 3 0 0 1 3-3z"
      fill="currentColor"
    />
  </Leaf>
</template>
