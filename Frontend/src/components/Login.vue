<template>
  aa
  <div
    class="fixed h-full w-full flex justify-center items-center top-0 bg-main bg-opacity-50"
    @click="disable_login"
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
      <input class="mr-1" type="checkbox" v-model="persistent" />
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
import { useNotes } from "./compositions";
import axios from "axios";
import { ref } from "vue";

export default {
  name: "Login",
  emits: ["close"],

  setup(props, { emit }) {
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
          if (resp.data.status == "ok") {
            wrong.value = "";
            emit('close')
          } else {
            wrong.value = resp.data.status;
          }
        })
        .catch((err) => {
          console.err(err);
          
        }).finally(() => requesting.value = false);
    };

    const disable_login = () => {
      if (!requesting.value) {
        emit("close");
      }
    };

    return {
      username,
      password,
      send_credentials,
      persistent,
      wrong,
      disable_login,
      requesting,
    };
  },
};
</script>