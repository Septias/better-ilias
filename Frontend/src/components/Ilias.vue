<template>
  <h1 class="text-5xl m-5">Better Ilias</h1>
  <div
    v-for="(child, index) in root_node.children"
    :key="index"
    class="ml-5 cursor-pointer"
  >
    <Folder
      :index="index"
      @set_invisible="handle_set_inivisible"
      @set_visible="handle_set_visible"
      :node="child"
    ></Folder>
  </div>
  <div class="right-0 top-0 fixed">
    <!--<div
      class="bg-light-main p-2 m-2 rounded cursor-pointer"
      @click="edit_visibility = !edit_visibility"
    >
      <svg width="1em" height="1em" viewBox="0 0 24 24">
        <path
          v-if="!edit_visibility"
          d="M11.5 18c3.989 0 7.458-2.224 9.235-5.5A10.498 10.498 0 0 0 11.5 7a10.498 10.498 0 0 0-9.235 5.5A10.498 10.498 0 0 0 11.5 18zm0-12a11.5 11.5 0 0 1 10.36 6.5A11.5 11.5 0 0 1 11.5 19a11.5 11.5 0 0 1-10.36-6.5A11.5 11.5 0 0 1 11.5 6zm0 2a4.5 4.5 0 1 1 0 9a4.5 4.5 0 0 1 0-9zm0 1a3.5 3.5 0 1 0 0 7a3.5 3.5 0 0 0 0-7z"
          fill="currentColor"
        ></path>
        <path
          v-else
          d="M2.543 4.707L3.25 4L20 20.75l-.707.707l-3.348-3.348c-1.367.574-2.87.891-4.445.891a11.5 11.5 0 0 1-10.36-6.5a11.55 11.55 0 0 1 4.374-4.821L2.543 4.707zM11.5 18c1.293 0 2.531-.234 3.675-.661l-1.129-1.128A4.5 4.5 0 0 1 7.79 9.954L6.244 8.408a10.55 10.55 0 0 0-3.98 4.092A10.498 10.498 0 0 0 11.5 18zm9.235-5.5A10.498 10.498 0 0 0 11.5 7a10.49 10.49 0 0 0-3.305.53l-.783-.782A11.474 11.474 0 0 1 11.5 6a11.5 11.5 0 0 1 10.36 6.5a11.55 11.55 0 0 1-4.068 4.628l-.724-.724a10.552 10.552 0 0 0 3.667-3.904zM11.5 8a4.5 4.5 0 0 1 3.904 6.74l-.74-.74A3.5 3.5 0 0 0 10 9.336l-.74-.74A4.48 4.48 0 0 1 11.5 8zM8 12.5a3.5 3.5 0 0 0 5.324 2.988l-4.812-4.812A3.484 3.484 0 0 0 8 12.5z"
          fill="currentColor"
        ></path>
      </svg>
    </div>-->
    <div class="bg-light-main p-2 m-2 rounded cursor-pointer" @click="update">
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
        ></path>
        <path
          v-else
          d="M12 5v12.25L17.25 12l.75.664l-6.5 6.5l-6.5-6.5l.75-.664L11 17.25V5h1z"
          fill="currentColor"
        ></path>
      </svg>
    </div>
  </div>
  <div
    class="fixed h-full w-full flex justify-center items-center top-0 bg-main bg-opacity-50"
    @click="disable_login"
    v-show="login"
  >
    <form
      class="bg-main rounded-xl border-2 border-accent p-4 custom_form text-xl"
      @submit.prevent="send_credentials"
      @click.stop=""
    >
      <p v-if="wrong" class="text-sm text-accent">{{ wrong }}</p>
      <label>Benutzername</label>
      <input v-model="username" autocomplete="username" class="block w-full" />
      <label>Passwort</label>
      <input
        v-model="password"
        autocomplete="current-password"
        class="block w-full"
        type="password"
      />
      <input class="mr-1" type="checkbox" v-model="persistent"/>
      <p class="inline-block text-sm">Angemeldet bleiben</p>
      <button
        type="submit"
        class="button px-2 rounded float-right"
        :class="requesting ? 'bg-gray-600' : 'bg-accent'"
        :disabled="requesting"
      >
        Ok!
      </button>
    </form>
  </div>
</template>

<style lang="sass" scoped>
.custom_form
  label
    @apply text-accent

  input
    @apply bg-light-main
    @apply p-1
    @apply mb-3
</style>

<script>
import { defineComponent, ref } from "vue";
import axios from "axios";
import { useVisibility } from "./compositions";

export default defineComponent({
  name: "Ilias",
  async setup() {
    let resp = await axios.get("/api/node");
    let data = resp.data;
    let root_node = ref(data);

    function handle_set_visible(path) {
      let node = root_node.value;
      for (let index of path.reverse()) {
        node = node.children[index];
        node.visible = true;
      }
      node.visible = true;
    }

    function handle_set_inivisible(path) {
      let node = root_node.value;
      for (let index of path.reverse()) {
        node = node.children[index];
      }
      node.visible = false;
    }

    let { edit_visibility } = useVisibility();

    let updating = ref(false);
    const update = () => {
      let start = Date.now();
      if (!updating.value){
        updating.value = true;
        axios
          .get("api/update")
          .then((resp) => {
            updating.value = false;
            if (resp.data.status == "set_token") {
              login.value = true;
              return;
            }
            root_node.value = resp.data.node;
            console.log("updated after", Date.now() - start, "ms");
          })
          .catch((e) => {
            console.error(e);
            updating.value = false;
          });
      }
    };

    update();

    const login = ref(false);

    const username = ref("");
    const password = ref("");
    const persistent = ref(false);
    const wrong = ref("");
    const requesting = ref(false);
    const send_credentials = () => {
      requesting.value = true;
      wrong.value = "";
      axios
        .post("api/credentials", {
          username: username.value,
          password: password.value,
          persistent: persistent.value,
        })
        .then((resp) => {
          requesting.value = false;
          console.log(resp.data.status);
          if (resp.data.status == "ok") {
            wrong.value = "";
            login.value = false;
            update()
          } else {
            wrong.value = resp.data.status;
          }
        })
        .catch((err) => {
          console.err(err);
          requesting.value = false;
        });
    };

    const disable_login = () => {
      if (!requesting.value) {
        login.value = !login.value;
      }
    };

    return {
      root_node,
      handle_set_inivisible,
      handle_set_visible,
      edit_visibility,
      updating,
      update,
      login,
      username,
      password,
      send_credentials,
      persistent,
      wrong,
      requesting,
      disable_login,
    };
  },
});
</script>
