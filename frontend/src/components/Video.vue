<script setup lang="ts">
import { edit_visibility } from '~/composables/visibility'

defineProps({
  node: {
    type: Object,
    required: true,
  },
})

async function open() {
  await invoke_log('open', { path: `https://ilias.uni-freiburg.de/${props.node.uri}` })
}
</script>

<template>
  <Leaf color="text-yellow-400" :node="node">
    <template #default>
      icon
    </template>
    <template #body>
      <a
        v-if="!edit_visibility"
        @click="open"
      >{{ node.title }}</a>
      <template v-else>
        {{ node.title }}
      </template>
    </template>
  </Leaf>
</template>
