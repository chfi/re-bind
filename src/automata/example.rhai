export const RIGHT_SHOULDER_DOWN = In_r_down;
export const RIGHT_SHOULDER_UP = In_r_up;
export const A_DOWN = In_a_down;
export const A_UP = In_a_up;

export const state = S_init;

fn machine() {
    silent(S_init, In_r_down, S_r);
    emit(S_init, In_a_down, S_a, Out_a);

    silent(S_r, In_r_up, S_init);
    silent(S_a, In_a_up, S_init);

    emit(S_r, In_a_down, S_ra, Out_ra);
    silent(S_ra, In_a_up, S_r);
}