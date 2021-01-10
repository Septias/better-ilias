import { computed, ref } from "vue"


let edit_visibility = ref(false)

export function useVisibility() {

    return { edit_visibility }
}