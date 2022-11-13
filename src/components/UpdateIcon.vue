<script>
import { ref } from 'vue'

export default {
  name: 'UpdateIcon',
  props: {
    updating: Boolean,
  },
}
</script>

<template>
  <div class="bg-light-main p-2 m-2 rounded cursor-pointer">
    <svg
      width="1em"
      height="1em"
      viewBox="0 0 24 24"
      :class="{ 'animate-spin': updating }"
    >
      <path
        v-if="updating"
        d="M11.5 4A8.5 8.5 0 0 0 3 12.5H2A9.5 9.5 0 0 1 11.5 3v1z"
        fill="currentColor"
      />
      <path
        v-else
        d="M12 5v12.25L17.25 12l.75.664l-6.5 6.5l-6.5-6.5l.75-.664L11 17.25V5h1z"
        fill="currentColor"
      />
    </svg>
  </div>
</template>
