<script setup lang="ts">
import { edit_visibility } from '~/composables/visibility'

defineProps({
  node: {
    type: Object,
    required: true,
  },
})
</script>

<template>
  <Leaf :node="node" color="text-yellow-500">
    <template #default>
      <path
        d="M2 16.586L5.586 13H15a2 2 0 0 0 2-2V6a2 2 0 0 0-2-2H4a2 2 0 0 0-2 2v10.586zM2 18H1V6a3 3 0 0 1 3-3h11a3 3 0 0 1 3 3v5a3 3 0 0 1-3 3H6l-4 4zm19 2.586V10a2 2 0 0 0-2-2V7a3 3 0 0 1 3 3v12h-1l-4-4H8a3 3 0 0 1-2.761-1.825l.797-.797A2 2 0 0 0 8 17h9.414L21 20.586z"
        fill="currentColor"
      />
    </template>
    <template #body>
      <a
        v-if="!edit_visibility"
        :href="`https://ilias.uni-freiburg.de/${node.uri}`"
      >{{ node.title }}</a>
      <template v-else>
        {{ node.title }}
      </template>
    </template>
  </Leaf>
</template>

