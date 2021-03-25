<template>
  <Leaf :node="node" color="text-blue-500" @dblclick="open_file">
    <path
      d="M14 11a3 3 0 0 1-3-3V4H7a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2h9a2 2 0 0 0 2-2v-8h-4zm-2-3a2 2 0 0 0 2 2h3.586L12 4.414V8zM7 3h5l7 7v9a3 3 0 0 1-3 3H7a3 3 0 0 1-3-3V6a3 3 0 0 1 3-3z"
      fill="currentColor"
    ></path>
  </Leaf>
</template>

<script>
import axios from "axios";
import { useVisibility } from "./compositions";
export default {
  name: "Forum",
  props: {
    node: Object,
  },

  setup(props) {
    function open_file() {
      axios.post("/api/open/", {
        path: props.node.breed.File.path,
      });
    }
    return { open_file };
  },
};
</script>