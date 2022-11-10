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
  <Leaf color="text-purple-500" :node="node">
    <template #default>
      <div class="i-carbon-settings" />
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
