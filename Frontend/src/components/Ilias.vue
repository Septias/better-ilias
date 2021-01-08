<template>
  <h1 class="text-5xl m-5">Better Ilias</h1>
  <Folder class="ml-5" v-for="child in root_node.children" :node="child"></Folder>
</template>

<script lang="ts">
import { defineComponent, ref } from "vue";
import axios from "axios";

export default defineComponent({
  name: "App",
  components: {
    Node,
  },
  async setup() {
    let resp = await axios.get("/api/node");
    let root_node = ref(resp.data);
    return { root_node };
  },
});
</script>
