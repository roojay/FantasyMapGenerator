//! glibc 兼容的随机数生成器
//!
//! 实现了与 glibc 的 rand() 函数完全相同的随机数生成算法。
//! 这确保了 Rust 版本与 C++ 版本在给定相同种子时生成完全相同的随机数序列。
//!
//! # 为什么需要自定义随机数生成器
//! 标准库的随机数生成器在不同平台和版本之间可能有差异。
//! 为了保证地图生成的可重现性（相同种子生成相同地图），
//! 我们需要使用与 C++ 版本完全相同的随机数算法。
//!
//! # 算法说明
//! glibc 使用 TYPE_3 线性反馈移位寄存器（LFSR）：
//! - 状态数组大小：31
//! - 反馈多项式：x^31 + x^3 + 1
//! - 输出范围：[0, 2^31-1]
//!
//! # 参考来源
//! - glibc 源码: stdlib/random_r.c
//! - 原始 C++ 实现: src/rand.h, src/rand.cpp

/// glibc rand() 的精确复制实现
///
/// 使用 TYPE_3 算法（degree=31, separation=3），
/// 通过 srand(seed) 初始化。
///
/// # 算法特性
/// - 周期长度：2^31 - 1
/// - 状态空间：31 个 32 位整数
/// - 反馈位置：fptr 和 rptr（间隔为 3）
///
/// # 兼容性
/// 此实现与 glibc 的 rand() 完全兼容，
/// 相同的种子会产生相同的随机数序列。
pub struct GlibcRand {
    /// 随机数生成器的状态数组（31 个元素）
    state: [u32; 31],
    
    /// 前向指针（feed pointer）
    fptr: usize,
    
    /// 后向指针（rear pointer）
    rptr: usize,
}

impl GlibcRand {
    /// 创建一个新的随机数生成器
    ///
    /// # 初始化过程
    /// 1. 使用 Park-Miller LCG 初始化状态数组
    /// 2. 执行 310 次预热迭代
    /// 3. 设置初始指针位置
    ///
    /// # 为什么需要预热
    /// 预热迭代确保初始状态充分混合，
    /// 避免种子的简单模式直接影响输出。
    ///
    /// # 参数
    /// * `seed` - 随机数种子（0 会被转换为 1）
    ///
    /// # 参考来源
    /// - glibc: stdlib/random_r.c, __srandom_r()
    pub fn new(seed: u32) -> Self {
        // glibc 规则：种子为 0 时使用 1
        let seed = if seed == 0 { 1 } else { seed };

        let mut state = [0u32; 31];
        
        // ===================================
        // 1. 使用 Park-Miller LCG 初始化状态
        // ===================================
        // LCG 公式: X(n+1) = (16807 * X(n)) mod (2^31 - 1)
        // 使用 Schrage 方法避免溢出
        state[0] = seed;
        for i in 1..31 {
            let prev = state[i - 1] as i32;
            let hi = prev / 127773;
            let lo = prev % 127773;
            let mut word = 16807 * lo - 2836 * hi;
            if word < 0 { word += 2147483647; }
            state[i] = word as u32;
        }
        
        // ===================================
        // 2. 预热：310 次迭代
        // ===================================
        let mut fptr = 3usize;
        let mut rptr = 0usize;
        for _ in 0..310 {
            state[fptr] = state[fptr].wrapping_add(state[rptr]);
            fptr = (fptr + 1) % 31;
            rptr = (rptr + 1) % 31;
        }
        
        // 预热后指针位置：fptr=3, rptr=0
        GlibcRand { state, fptr: 3, rptr: 0 }
    }

    /// 生成下一个随机整数
    ///
    /// # 算法
    /// 1. 更新状态：state[fptr] += state[rptr]
    /// 2. 提取结果：右移 1 位并屏蔽最高位
    /// 3. 移动指针
    ///
    /// # 返回
    /// 范围 [0, 0x7fffffff] 的随机整数
    ///
    /// # 参考来源
    /// - glibc: stdlib/random_r.c, __random_r()
    pub fn rand(&mut self) -> i32 {
        // 更新状态（加法反馈）
        self.state[self.fptr] = self.state[self.fptr].wrapping_add(self.state[self.rptr]);
        
        // 提取结果：右移 1 位，保留 31 位
        let result = (self.state[self.fptr] >> 1) & 0x7fffffff;
        
        // 移动指针（循环）
        self.fptr = (self.fptr + 1) % 31;
        self.rptr = (self.rptr + 1) % 31;
        
        result as i32
    }

    /// 生成指定范围内的随机浮点数
    ///
    /// # 公式
    /// ```text
    /// result = min + rand() / (RAND_MAX / (max - min))
    /// ```
    ///
    /// # 参数
    /// * `min` - 最小值（包含）
    /// * `max` - 最大值（不包含）
    ///
    /// # 返回
    /// 范围 [min, max) 的随机浮点数
    pub fn random_double(&mut self, min: f64, max: f64) -> f64 {
        min + (self.rand() as f64) / (0x7fffffff as f64 / (max - min))
    }

    /// 生成指定范围内的随机整数
    ///
    /// # 公式
    /// ```text
    /// result = min + (rand() % (max - min))
    /// ```
    ///
    /// # 参数
    /// * `min` - 最小值（包含）
    /// * `max` - 最大值（不包含）
    ///
    /// # 返回
    /// 范围 [min, max) 的随机整数
    pub fn random_range(&mut self, min: i32, max: i32) -> i32 {
        min + (self.rand() % (max - min))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glibc_rand_sequence() {
        // Verify exact glibc rand() values match C reference implementation
        // Reference: srand(42), skip 1000, then read 5 values
        // Verified against: gcc -o test_rand test_rand.c && ./test_rand
        let mut rng = GlibcRand::new(42);
        for _ in 0..1000 {
            rng.rand();
        }
        assert_eq!(rng.rand(), 1963050744, "rand()[1000] mismatch");
        assert_eq!(rng.rand(), 30553106,   "rand()[1001] mismatch");
        assert_eq!(rng.rand(), 957990501,  "rand()[1002] mismatch");
        assert_eq!(rng.rand(), 953383689,  "rand()[1003] mismatch");
        assert_eq!(rng.rand(), 348269264,  "rand()[1004] mismatch");
    }

    #[test]
    fn test_glibc_rand_seed_zero_treated_as_one() {
        // glibc treats seed=0 as seed=1
        let mut rng0 = GlibcRand::new(0);
        let mut rng1 = GlibcRand::new(1);
        assert_eq!(rng0.rand(), rng1.rand(), "seed=0 should equal seed=1");
    }
}
