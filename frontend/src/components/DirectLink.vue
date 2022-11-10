<script setup lang="ts">
import { edit_visibility } from '~/composables/visibility'
import { invoke_log } from '~/utils'

const props = defineProps({
  node: {
    type: Object,
    required: true,
  },
})

async function open() {
  await invoke_log('open', { path: props.node.uri })
}
</script>

<template>
  <Leaf color="text-green-500" :node="node">
    <template #default>
      <path
        d="M8 13v-1h7v1H8zm7.5-6a5.5 5.5 0 1 1 0 11H13v-1h2.5a4.5 4.5 0 1 0 0-9H13V7h2.5zm-8 11a5.5 5.5 0 1 1 0-11H10v1H7.5a4.5 4.5 0 1 0 0 9H10v1H7.5z"
        fill="currentColor"
      />
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
