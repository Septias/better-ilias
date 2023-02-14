<script setup lang="ts">
import { invoke } from '@tauri-apps/api'
import { ref } from 'vue'
import NProgress from 'nprogress'

const username = useStorage('un', '')
const password = useStorage('pw', '')
const wrong = ref('')
const requesting = ref(false)

const router = useRouter()
const login = async () => {
  requesting.value = true
  wrong.value = ''

  try {
    NProgress.start()
    await invoke('login', {
      creds: {
        name: username.value,
        pw: password.value,
      },
    })
    router.push('/')
  }
  catch (e) {
    console.error(e)
    wrong.value = 'Wrong username or password'
  }
  finally {
    requesting.value = false
    NProgress.done()
  }
}
</script>

<template>
  <div class="fixed h-full w-full flex justify-center items-center top-0 bg-main text-white">
    <form class="bg-main rounded-xl border-2 border-accent p-4 custom_form text-xl" @submit.prevent="login"
      @click.stop="">
      <p v-if="wrong" class="text-sm text-accent">
        {{ wrong }}
      </p>
      <label>Benutzername</label>
      <input v-model="username" autocomplete="username" class="block w-full bg-light_main rounded">
      <label>Passwort</label>
      <input v-model="password" autocomplete="current-password" class="block w-full bg-light_main rounded">
      <button type="submit" class="button px-2 rounded float-right" :class="requesting ? 'bg-gray-600' : 'bg-accent'"
        :disabled="requesting" @click="login">
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
