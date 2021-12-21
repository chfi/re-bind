export let state = S_init;

silent(S_init, In_RMB_down, S_rmb);
emit(S_rmb, In_RMB_up, S_init, O_ctx_open)

emit(S_rmb, In_mouse_move, S_pan, O_mode_start);
emit(S_pan, In_RMB_up, S_init, O_mode_end);

emit(S_pan, In_LMB_down, S_select, O_pan_select);
emit(S_select, In_LMB_down, S_pan, O_select_pan);