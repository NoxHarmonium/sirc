function tex_escape(s,    i, c, out) {
	out = ""
	for (i = 1; i <= length(s); i++) {
		c = substr(s, i, 1)
		if (c == "\\") {
			out = out "\\textbackslash{}"
		} else if (c == "#" || c == "%" || c == "_" || c == "&") {
			out = out "\\" c
		} else {
			out = out c
		}
	}
	return out
}

function pow2(n) {
	return 2 ^ n
}

function hex_value(c) {
	c = toupper(c)
	if (c >= "0" && c <= "9") {
		return c + 0
	}
	return index("ABCDEF", c) + 9
}

function hex_to_number(s,    i, start, n) {
	start = substr(s, 1, 2) == "0x" || substr(s, 1, 2) == "0X" ? 3 : 1
	n = 0
	for (i = start; i <= length(s); i++) {
		n = (n * 16) + hex_value(substr(s, i, 1))
	}
	return n
}

function bits(value, offset, width) {
	return int(value / pow2(offset)) % pow2(width)
}

function hex_field(value, width) {
	return sprintf("0x%0*X", width, value)
}

function bin_field(value, width, group,    i, out, bit) {
	out = ""
	for (i = width - 1; i >= 0; i--) {
		bit = int(value / pow2(i)) % 2
		out = out bit
		if (group > 0 && i > 0 && i % group == 0) {
			out = out " "
		}
	}
	return "(" out ")"
}

function bin_plain(value, width,    i, out, bit) {
	out = ""
	for (i = width - 1; i >= 0; i--) {
		bit = int(value / pow2(i)) % 2
		out = out bit
	}
	return out
}

function emit(line) {
	print line > out_file
}

function close_table() {
	if (open) {
		emit("\\bottomrule")
		emit("\\end{tabular}")
		emit("\\caption{" caption "}")
		emit("\\label{" label "}")
		emit("\\end{table}")
		close(out_file)
		open = 0
	}
}

function label_for_section(section) {
	if (section == "short-immediate") {
		return "short-imm-format-examples"
	}
	return section "-format-examples"
}

function open_table(section, title) {
	emit("\\begin{table}[H]")
	emit("\\centering")
	if (section == "immediate") {
		emit("\\small")
		emit("\\begin{tabular}{lcccccc}")
		emit("\\toprule")
		emit("\\textbf{Instruction} & \\textbf{Opcode} & \\textbf{Reg} & \\textbf{Immediate} & \\textbf{AF} & \\textbf{Cond} & \\textbf{Hex} \\\\")
	} else if (section == "short-immediate") {
		emit("\\tiny")
		emit("\\begin{tabular}{lccccccccc}")
		emit("\\toprule")
		emit("\\textbf{Instruction} & \\textbf{Op} & \\textbf{Reg} & \\textbf{Imm} & \\textbf{SO} & \\textbf{ST} & \\textbf{SA} & \\textbf{AF} & \\textbf{Cond} & \\textbf{Hex} \\\\")
	} else if (section == "register") {
		emit("\\tiny")
		emit("\\begin{tabular}{lcccccccccc}")
		emit("\\toprule")
		emit("\\textbf{Instruction} & \\textbf{Op} & \\textbf{R1} & \\textbf{R2} & \\textbf{R3} & \\textbf{SO} & \\textbf{ST} & \\textbf{SA} & \\textbf{AF} & \\textbf{Cond} & \\textbf{Hex} \\\\")
	} else {
		print "Unknown manual section: " section > "/dev/stderr"
		exit 1
	}
	emit("\\midrule")
	caption = title
	label = "tab:" label_for_section(section)
	row_count = 0
	open = 1
}

function emit_immediate_row(instruction, word, encoded,    op, reg, imm, af, cond) {
	op = bits(encoded, 26, 6)
	reg = bits(encoded, 22, 4)
	imm = bits(encoded, 6, 16)
	af = bits(encoded, 4, 2)
	cond = bits(encoded, 0, 4)

	emit("\\texttt{" tex_escape(instruction) "} & " hex_field(op, 2) " & " hex_field(reg, 1) " & " hex_field(imm, 4) " & 0b" bin_plain(af, 2) " & " hex_field(cond, 1) " & \\texttt{" word "} \\\\")
	emit("& " bin_field(op, 6, 0) " & " bin_field(reg, 4, 0) " & " bin_field(imm, 16, 4) " & " bin_field(af, 2, 0) " & " bin_field(cond, 4, 0) " & \\\\")
}

function emit_short_immediate_row(instruction, word, encoded,    op, reg, imm, so, st, sa, af, cond) {
	op = bits(encoded, 26, 6)
	reg = bits(encoded, 22, 4)
	imm = bits(encoded, 14, 8)
	so = bits(encoded, 13, 1)
	st = bits(encoded, 10, 3)
	sa = bits(encoded, 6, 4)
	af = bits(encoded, 4, 2)
	cond = bits(encoded, 0, 4)

	emit("\\texttt{" tex_escape(instruction) "} & " hex_field(op, 2) " & " hex_field(reg, 1) " & " hex_field(imm, 2) " & " so " & " bin_plain(st, 3) " & " hex_field(sa, 1) " & 0b" bin_plain(af, 2) " & " hex_field(cond, 1) " & \\texttt{" word "} \\\\")
	emit("& " bin_field(op, 6, 0) " & " bin_field(reg, 4, 0) " & " bin_field(imm, 8, 0) " & " bin_field(so, 1, 0) " & " bin_field(st, 3, 0) " & " bin_field(sa, 4, 0) " & " bin_field(af, 2, 0) " & " bin_field(cond, 4, 0) " & \\\\")
}

function emit_register_row(instruction, word, encoded,    op, r1, r2, r3, so, st, sa, af, cond) {
	op = bits(encoded, 26, 6)
	r1 = bits(encoded, 22, 4)
	r2 = bits(encoded, 18, 4)
	r3 = bits(encoded, 14, 4)
	so = bits(encoded, 13, 1)
	st = bits(encoded, 10, 3)
	sa = bits(encoded, 6, 4)
	af = bits(encoded, 4, 2)
	cond = bits(encoded, 0, 4)

	emit("\\texttt{" tex_escape(instruction) "} & " hex_field(op, 2) " & " hex_field(r1, 1) " & " hex_field(r2, 1) " & " hex_field(r3, 1) " & " so " & " bin_plain(st, 3) " & " hex_field(sa, 1) " & 0b" bin_plain(af, 2) " & " hex_field(cond, 1) " & \\texttt{" word "} \\\\")
	emit("& " bin_field(op, 6, 0) " & " bin_field(r1, 4, 0) " & " bin_field(r2, 4, 0) " & " bin_field(r3, 4, 0) " & " bin_field(so, 1, 0) " & " bin_field(st, 3, 0) " & " bin_field(sa, 4, 0) " & " bin_field(af, 2, 0) " & " bin_field(cond, 4, 0) " & \\\\")
}

function emit_example_row(instruction, word,    encoded) {
	encoded = hex_to_number(word)
	if (row_count > 0) {
		emit("\\midrule")
	}
	if (section == "immediate") {
		emit_immediate_row(instruction, word, encoded)
	} else if (section == "short-immediate") {
		emit_short_immediate_row(instruction, word, encoded)
	} else if (section == "register") {
		emit_register_row(instruction, word, encoded)
	} else {
		print "Manual example before manual section: " instruction > "/dev/stderr"
		exit 1
	}
	row_count++
}

BEGIN {
	if (word_file == "" || out_dir == "") {
		print "word_file and out_dir are required" > "/dev/stderr"
		exit 1
	}
}

/^manual-section / {
	close_table()
	split($0, parts, " ")
	section = parts[2]
	out_file = out_dir "/" parts[2] "-format-encodings.tex"
	open_table(section, substr($0, length(parts[1]) + length(parts[2]) + 3))
	next
}

/^manual-example / {
	if ((getline word < word_file) <= 0) {
		print "Missing encoded word for " $0 > "/dev/stderr"
		exit 1
	}
	instruction = substr($0, length("manual-example ") + 1)
	emit_example_row(instruction, word)
	next
}

END {
	close_table()
	close(word_file)
}
