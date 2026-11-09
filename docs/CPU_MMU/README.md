# 해외유튜브 영상
- [250303Page Tables and MMU: How Virtual Memory Actually Works Behind the Scenes (Animation) |@BitLemonSoftware](https://youtu.be/B6tJxvYBNrU?si=dJMODHcANl1nyrm8)

# page-tables
- https://github.com/ljskernel/linux-vm-notes/blob/master/sections/page-tables.md

# MMU & TLB
- https://www.minzkn.com/linuxkernel/pages/mmu-tlb.html
- MMU (Memory Management Unit)와 TLB (Translation Lookaside Buffer)는 가상 메모리(Virtual Memory)의 하드웨어 핵심입니다. x86-64/ARM64 주소 변환(Address Translation), 4/5단계 페이지 테이블(Page Table) 워크, PTE 비트필드, TLB 내부 구조, mmu_gather 배칭, 페이지 폴트(Page Fault) 핸들링, COW(Copy-on-Write), KPTI(Kernel Page Table Isolation), Huge Pages/THP TLB 최적화, PCID/ASID, 커널 주소 공간(Address Space) 레이아웃, TLB 관련 보안 취약점(Vulnerability)과 완화 기법, 실전 성능 분석 사례까지 종합적으로 다룹니다.

## (MMU)Memory management unit
- https://en.wikipedia.org/wiki/Memory_management_unit

## Translation Lookaside Buffer(TLB)
- https://en.wikipedia.org/wiki/Translation_lookaside_buffer
