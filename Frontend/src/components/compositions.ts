import { computed, ref } from "vue"


let edit_visibility = ref(true)

export function useVisibility() {
    return { edit_visibility }
}