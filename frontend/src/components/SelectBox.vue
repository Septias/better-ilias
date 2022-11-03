<template>
  <input class="cursor-pointer" type="checkbox" :checked="checked" />
</template>

<script lang="ts">
import { defineComponent } from "vue";

export default defineComponent({
  name: "SelectBox",
  props: {
    checked: Boolean,
  },
});
</script>