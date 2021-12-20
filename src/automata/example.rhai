export RIGHT_SHOULDER = In_r;
export A = In_a;

export state = S_init;

fn machine() {
    silent(S_init, In_r.down, S_r);
    emit(S_init, In_a.down, S_a, Out_a);

    silent(S_r, In_r.up, S_init);
    silent(S_a, In_a.up, S_init);

    emit(S_r, In_a.down, S_ra, Out_ra);
    silent(S_ra, In_a.up, S_r);
}